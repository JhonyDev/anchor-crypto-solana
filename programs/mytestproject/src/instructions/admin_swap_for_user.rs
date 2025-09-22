use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::{Vault, UserVaultAccount, UserTokenAccount, TokenVault};
use crate::constants::{TOKEN_VAULT_SEED, USER_TOKEN_ACCOUNT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct AdminSwapForUser<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [b"user_vault", target_user.key().as_ref()],
        bump = user_vault.bump,
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [USER_TOKEN_ACCOUNT_SEED, target_user.key().as_ref()],
        bump
    )]
    pub user_token_account: Account<'info, UserTokenAccount>,
    
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    #[account(
        mut,
        constraint = vault_wsol_ata.key() == token_vault.vault_wsol_ata
    )]
    pub vault_wsol_ata: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = vault_usdc_ata.key() == token_vault.vault_usdc_ata
    )]
    pub vault_usdc_ata: Account<'info, TokenAccount>,
    
    /// CHECK: The user for whom the swap is being executed
    pub target_user: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AdminSwapForUser>,
    sol_amount: u64,
    min_usdc_out: u64,
) -> Result<()> {
    let user_vault = &mut ctx.accounts.user_vault;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let token_vault = &ctx.accounts.token_vault;
    
    require!(
        user_vault.current_balance >= sol_amount,
        ErrorCode::InsufficientUserBalance
    );
    
    require!(
        token_vault.total_wsol >= sol_amount,
        ErrorCode::InsufficientWSOLBalance
    );
    
    if user_token_account.owner == Pubkey::default() {
        user_token_account.owner = ctx.accounts.target_user.key();
        user_token_account.bump = ctx.bumps.user_token_account;
        user_token_account.wsol_balance = 0;
        user_token_account.usdc_balance = 0;
        user_token_account.last_swap_timestamp = 0;
        user_token_account.total_swapped = 0;
    }
    
    // Mock swap rate for now (in production, use actual Orca swap)
    let swap_rate = 40_000_000; // 40 USDC per SOL
    let usdc_amount = sol_amount.checked_mul(swap_rate)
        .and_then(|v| v.checked_div(1_000_000))
        .ok_or(ErrorCode::MathOverflow)?;
    
    require!(
        usdc_amount >= min_usdc_out,
        ErrorCode::SlippageExceeded
    );
    
    // Update balances
    user_vault.current_balance = user_vault.current_balance.checked_sub(sol_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    user_token_account.wsol_balance = user_token_account.wsol_balance.checked_sub(sol_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    user_token_account.usdc_balance = user_token_account.usdc_balance.checked_add(usdc_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    user_token_account.total_swapped = user_token_account.total_swapped.checked_add(sol_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    let clock = Clock::get()?;
    user_token_account.last_swap_timestamp = clock.unix_timestamp;
    user_vault.last_transaction = clock.unix_timestamp;
    
    msg!("Admin {} executed swap of {} SOL for user {}", 
         ctx.accounts.authority.key(), 
         sol_amount,
         ctx.accounts.target_user.key());
    msg!("User received {} USDC", usdc_amount);
    
    Ok(())
}