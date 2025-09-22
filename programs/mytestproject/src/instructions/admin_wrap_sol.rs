use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, Mint, SyncNative};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{Vault, TokenVault, UserVaultAccount};
use crate::constants::{WSOL_MINT, TOKEN_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct AdminWrapSol<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
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
        seeds = [b"user_vault", target_user.key().as_ref()],
        bump = user_vault.bump,
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    /// CHECK: The user whose SOL is being wrapped
    pub target_user: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AdminWrapSol>, amount: u64) -> Result<()> {
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
    
    // Transfer SOL from vault PDA to wSOL ATA
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: vault_pda.to_account_info(),
            to: vault_wsol_ata.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(transfer_ctx, amount)?;
    
    // Sync native account to update wSOL balance
    let sync_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SyncNative {
            account: vault_wsol_ata.to_account_info(),
        },
    );
    token::sync_native(sync_ctx)?;
    
    // Update balances
    user_vault.current_balance = user_vault.current_balance.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    token_vault.total_wsol = token_vault.total_wsol.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    msg!("Admin {} wrapped {} lamports to wSOL for user {}", 
         ctx.accounts.authority.key(), 
         amount, 
         ctx.accounts.target_user.key());
    
    Ok(())
}