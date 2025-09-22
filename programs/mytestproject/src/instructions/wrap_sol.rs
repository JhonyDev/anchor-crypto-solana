use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, Mint, SyncNative};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{TokenVault, UserVaultAccount};
use crate::constants::{WSOL_MINT, TOKEN_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct WrapSol<'info> {
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    /// CHECK: This is the vault PDA that holds SOL
    #[account(
        mut,
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
    
    #[account(address = WSOL_MINT)]
    pub wsol_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump = user_vault.bump,
        has_one = owner @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    pub user: Signer<'info>,
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WrapSol>, amount: u64) -> Result<()> {
    let user_vault = &mut ctx.accounts.user_vault;
    let vault_pda = &ctx.accounts.vault_pda;
    let vault_wsol_ata = &ctx.accounts.vault_wsol_ata;
    let token_vault = &mut ctx.accounts.token_vault;
    
    require!(
        user_vault.current_balance >= amount,
        ErrorCode::InsufficientUserBalance
    );
    
    require!(
        vault_pda.get_lamports() >= amount,
        ErrorCode::InsufficientFunds
    );
    
    let vault_pda_seeds = &[
        b"vault_pda".as_ref(),
        &[ctx.bumps.vault_pda]
    ];
    let signer_seeds = &[&vault_pda_seeds[..]];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: vault_pda.to_account_info(),
            to: vault_wsol_ata.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(transfer_ctx, amount)?;
    
    let sync_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SyncNative {
            account: vault_wsol_ata.to_account_info(),
        },
    );
    token::sync_native(sync_ctx)?;
    
    user_vault.current_balance = user_vault.current_balance.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    token_vault.total_wsol = token_vault.total_wsol.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    msg!("Wrapped {} lamports to wSOL for user {}", amount, ctx.accounts.user.key());
    
    Ok(())
}