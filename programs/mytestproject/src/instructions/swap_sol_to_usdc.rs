use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::{TokenVault, SwapState};
use crate::constants::{WSOL_MINT, USDC_MINT, TOKEN_VAULT_SEED, ORCA_WHIRLPOOL_PROGRAM_ID};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct SwapSolToUsdc<'info> {
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + 8 + 8 + 8 + 8,
        seeds = [b"swap_state"],
        bump
    )]
    pub swap_state: Account<'info, SwapState>,
    
    /// CHECK: This is the vault PDA that owns token accounts
    #[account(
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: AccountInfo<'info>,
    
    #[account(
        mut,
        associated_token::mint = wsol_mint,
        associated_token::authority = vault_pda,
    )]
    pub vault_wsol_ata: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_pda,
    )]
    pub vault_usdc_ata: Account<'info, TokenAccount>,
    
    #[account(address = WSOL_MINT)]
    pub wsol_mint: Account<'info, Mint>,
    
    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
    
    /// CHECK: Orca Whirlpool program
    #[account(address = ORCA_WHIRLPOOL_PROGRAM_ID)]
    pub whirlpool_program: AccountInfo<'info>,
    
    /// CHECK: Whirlpool state account
    pub whirlpool: AccountInfo<'info>,
    
    /// CHECK: Token vault A (wSOL) for the pool
    pub token_vault_a: AccountInfo<'info>,
    
    /// CHECK: Token vault B (USDC) for the pool
    pub token_vault_b: AccountInfo<'info>,
    
    /// CHECK: Tick array accounts for price discovery
    pub tick_array_0: AccountInfo<'info>,
    
    /// CHECK: Tick array accounts for price discovery
    pub tick_array_1: AccountInfo<'info>,
    
    /// CHECK: Tick array accounts for price discovery
    pub tick_array_2: AccountInfo<'info>,
    
    /// CHECK: Oracle account for price
    pub oracle: AccountInfo<'info>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SwapSolToUsdc<'info> {
    pub fn swap_accounts(&self) -> Vec<AccountInfo<'info>> {
        vec![
            self.whirlpool_program.to_account_info(),
            self.token_program.to_account_info(),
            self.vault_pda.to_account_info(),
            self.whirlpool.to_account_info(),
            self.token_vault_a.to_account_info(),
            self.token_vault_b.to_account_info(),
            self.vault_wsol_ata.to_account_info(),
            self.vault_usdc_ata.to_account_info(),
            self.tick_array_0.to_account_info(),
            self.tick_array_1.to_account_info(),
            self.tick_array_2.to_account_info(),
            self.oracle.to_account_info(),
        ]
    }
}

pub fn handler(
    ctx: Context<SwapSolToUsdc>,
    amount_in: u64,
    minimum_amount_out: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
) -> Result<()> {
    let vault_wsol_amount = ctx.accounts.vault_wsol_ata.amount;
    let token_vault_total_wsol = ctx.accounts.token_vault.total_wsol;
    let token_vault_total_usdc = ctx.accounts.token_vault.total_usdc;
    
    require!(
        vault_wsol_amount >= amount_in,
        ErrorCode::InsufficientWSOLBalance
    );
    
    require!(
        token_vault_total_wsol >= amount_in,
        ErrorCode::InsufficientWSOLBalance
    );
    
    let vault_pda_seeds = &[
        b"vault_pda".as_ref(),
        &[ctx.bumps.vault_pda]
    ];
    let signer_seeds = &[&vault_pda_seeds[..]];
    
    msg!("Preparing to swap {} wSOL for USDC", amount_in);
    msg!("Minimum USDC out: {}", minimum_amount_out);
    
    let vault_pda_key = ctx.accounts.vault_pda.key();
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ORCA_WHIRLPOOL_PROGRAM_ID,
        accounts: ctx.accounts.swap_accounts()
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.key == &vault_pda_key,
                is_writable: acc.is_writable,
            })
            .collect(),
        data: {
            let mut data = Vec::with_capacity(1 + 8 + 8 + 16 + 1 + 1);
            data.push(0xf8);
            data.extend_from_slice(&amount_in.to_le_bytes());
            data.extend_from_slice(&minimum_amount_out.to_le_bytes());
            data.extend_from_slice(&sqrt_price_limit.to_le_bytes());
            data.push(amount_specified_is_input as u8);
            data.push(a_to_b as u8);
            data
        },
    };
    
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &ctx.accounts.swap_accounts(),
        signer_seeds,
    )?;
    
    ctx.accounts.vault_wsol_ata.reload()?;
    ctx.accounts.vault_usdc_ata.reload()?;
    
    let usdc_received = ctx.accounts.vault_usdc_ata.amount.saturating_sub(token_vault_total_usdc);
    
    require!(
        usdc_received >= minimum_amount_out,
        ErrorCode::SlippageExceeded
    );
    
    let token_vault = &mut ctx.accounts.token_vault;
    let swap_state = &mut ctx.accounts.swap_state;
    
    token_vault.total_wsol = token_vault_total_wsol.checked_sub(amount_in)
        .ok_or(ErrorCode::MathOverflow)?;
    token_vault.total_usdc = ctx.accounts.vault_usdc_ata.amount;
    
    swap_state.total_sol_swapped = swap_state.total_sol_swapped.checked_add(amount_in)
        .ok_or(ErrorCode::MathOverflow)?;
    swap_state.total_usdc_received = swap_state.total_usdc_received.checked_add(usdc_received)
        .ok_or(ErrorCode::MathOverflow)?;
    swap_state.swap_count = swap_state.swap_count.checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;
    
    if amount_in > 0 {
        swap_state.last_swap_price = usdc_received.checked_mul(1_000_000)
            .and_then(|v| v.checked_div(amount_in))
            .ok_or(ErrorCode::MathOverflow)?;
    }
    
    msg!("Swap completed: {} wSOL -> {} USDC", amount_in, usdc_received);
    msg!("Price: {} USDC per SOL", swap_state.last_swap_price);
    
    Ok(())
}