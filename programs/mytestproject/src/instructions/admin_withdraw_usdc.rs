use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{Vault, UserTokenAccount, TokenVault};
use crate::constants::{USDC_MINT, USER_TOKEN_ACCOUNT_SEED, TOKEN_VAULT_SEED};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct AdminWithdrawUsdc<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(
        mut,
        seeds = [USER_TOKEN_ACCOUNT_SEED, target_user.key().as_ref()],
        bump = user_token_account.bump,
    )]
    pub user_token_account: Account<'info, UserTokenAccount>,
    
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED],
        bump = token_vault.bump,
        has_one = authority @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    /// CHECK: This is the vault PDA that owns token accounts
    #[account(
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: AccountInfo<'info>,
    
    #[account(
        mut,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_pda,
    )]
    pub vault_usdc_ata: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = target_user,
    )]
    pub user_usdc_ata: Account<'info, TokenAccount>,
    
    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
    
    /// CHECK: The user to withdraw USDC for
    pub target_user: AccountInfo<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AdminWithdrawUsdc>, amount: u64) -> Result<()> {
    let user_token_account = &mut ctx.accounts.user_token_account;
    let token_vault = &mut ctx.accounts.token_vault;
    let vault_usdc_ata = &ctx.accounts.vault_usdc_ata;
    
    require!(
        user_token_account.usdc_balance >= amount,
        ErrorCode::InsufficientUserBalance
    );
    
    require!(
        vault_usdc_ata.amount >= amount,
        ErrorCode::InsufficientFunds
    );
    
    let vault_pda_seeds = &[
        b"vault_pda".as_ref(),
        &[ctx.bumps.vault_pda]
    ];
    let signer_seeds = &[&vault_pda_seeds[..]];
    
    // Transfer USDC from vault to user
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        token::Transfer {
            from: ctx.accounts.vault_usdc_ata.to_account_info(),
            to: ctx.accounts.user_usdc_ata.to_account_info(),
            authority: ctx.accounts.vault_pda.to_account_info(),
        },
        signer_seeds,
    );
    
    token::transfer(transfer_ctx, amount)?;
    
    // Update balances
    user_token_account.usdc_balance = user_token_account.usdc_balance.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    token_vault.total_usdc = token_vault.total_usdc.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    msg!("Admin {} withdrew {} USDC for user {}", 
         ctx.accounts.authority.key(), 
         amount, 
         ctx.accounts.target_user.key());
    
    Ok(())
}