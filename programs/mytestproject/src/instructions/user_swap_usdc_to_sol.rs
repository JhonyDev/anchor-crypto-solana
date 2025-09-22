use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::state::{UserVaultAccount, UserTokenAccount, TokenVault};
use crate::constants::{USER_TOKEN_ACCOUNT_SEED, TOKEN_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct UserSwapUsdcToSol<'info> {
    #[account(
        mut,
        seeds = [b"user_vault", user.key().as_ref()],
        bump = user_vault.bump,
        has_one = owner @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [USER_TOKEN_ACCOUNT_SEED, user.key().as_ref()],
        bump
    )]
    pub user_token_account: Account<'info, UserTokenAccount>,
    
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    #[account(
        mut,
        constraint = vault_usdc_ata.key() == token_vault.vault_usdc_ata
    )]
    pub vault_usdc_ata: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = vault_wsol_ata.key() == token_vault.vault_wsol_ata
    )]
    pub vault_wsol_ata: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<UserSwapUsdcToSol>,
    usdc_amount: u64,
    min_sol_out: u64,
) -> Result<()> {
    let user_vault = &mut ctx.accounts.user_vault;
    let user_token_account = &mut ctx.accounts.user_token_account;
    let token_vault = &ctx.accounts.token_vault;
    
    // Check user has enough USDC
    require!(
        user_token_account.usdc_balance >= usdc_amount,
        ErrorCode::InsufficientUserBalance
    );
    
    // Check vault has enough USDC
    require!(
        token_vault.total_usdc >= usdc_amount,
        ErrorCode::InsufficientUserBalance
    );
    
    // Initialize user token account if needed
    if user_token_account.owner == Pubkey::default() {
        user_token_account.owner = ctx.accounts.user.key();
        user_token_account.bump = ctx.bumps.user_token_account;
        user_token_account.wsol_balance = 0;
        user_token_account.usdc_balance = 0;
        user_token_account.last_swap_timestamp = 0;
        user_token_account.total_swapped = 0;
    }
    
    // Mock swap rate for now (in production, use actual Orca swap)
    // 1 USDC = 0.025 SOL (inverse of 40 USDC per SOL)
    let swap_rate = 25_000_000; // 0.025 SOL per USDC
    let sol_amount = usdc_amount.checked_mul(swap_rate)
        .and_then(|v| v.checked_div(1_000_000_000))
        .ok_or(ErrorCode::MathOverflow)?;
    
    require!(
        sol_amount >= min_sol_out,
        ErrorCode::SlippageExceeded
    );
    
    // Update user token account
    user_token_account.usdc_balance = user_token_account.usdc_balance.checked_sub(usdc_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    user_token_account.wsol_balance = user_token_account.wsol_balance.checked_add(sol_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    // Update user vault SOL balance
    user_vault.current_balance = user_vault.current_balance.checked_add(sol_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    // Track the swap
    user_token_account.total_swapped = user_token_account.total_swapped.checked_add(usdc_amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    let clock = Clock::get()?;
    user_token_account.last_swap_timestamp = clock.unix_timestamp;
    user_vault.last_transaction = clock.unix_timestamp;
    
    msg!("User {} swapped {} USDC for {} SOL", 
         ctx.accounts.user.key(), 
         usdc_amount, 
         sol_amount);
    
    Ok(())
}