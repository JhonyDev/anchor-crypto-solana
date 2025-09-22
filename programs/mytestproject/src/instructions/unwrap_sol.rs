use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, CloseAccount};
use crate::state::{TokenVault, UserVaultAccount};
use crate::constants::{WSOL_MINT, TOKEN_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct UnwrapSol<'info> {
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    /// CHECK: This is the vault PDA that will receive SOL
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
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UnwrapSol>, amount: u64) -> Result<()> {
    let user_vault = &mut ctx.accounts.user_vault;
    let vault_wsol_ata = &ctx.accounts.vault_wsol_ata;
    let token_vault = &mut ctx.accounts.token_vault;
    let vault_pda = &ctx.accounts.vault_pda;
    
    require!(
        vault_wsol_ata.amount >= amount,
        ErrorCode::InsufficientWSOLBalance
    );
    
    require!(
        token_vault.total_wsol >= amount,
        ErrorCode::InsufficientWSOLBalance
    );
    
    let vault_pda_seeds = &[
        b"vault_pda".as_ref(),
        &[ctx.bumps.vault_pda]
    ];
    let signer_seeds = &[&vault_pda_seeds[..]];
    
    if vault_wsol_ata.amount == amount {
        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: vault_wsol_ata.to_account_info(),
                destination: vault_pda.to_account_info(),
                authority: vault_pda.to_account_info(),
            },
            signer_seeds,
        );
        token::close_account(close_ctx)?;
    } else {
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: vault_wsol_ata.to_account_info(),
                to: vault_pda.to_account_info(),
                authority: vault_pda.to_account_info(),
            },
            signer_seeds,
        );
        token::transfer(transfer_ctx, amount)?;
    }
    
    user_vault.current_balance = user_vault.current_balance.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    token_vault.total_wsol = token_vault.total_wsol.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    msg!("Unwrapped {} wSOL to lamports for user {}", amount, ctx.accounts.user.key());
    
    Ok(())
}