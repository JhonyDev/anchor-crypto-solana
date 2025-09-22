use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use crate::state::{Vault, UserVaultAccount};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    /// CHECK: This is the vault PDA that holds funds
    #[account(
        mut,
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: AccountInfo<'info>,
    
    #[account(
        init_if_needed,
        payer = depositor,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"user_vault", depositor.key().as_ref()],
        bump
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    let from = &ctx.accounts.depositor;
    let to = &ctx.accounts.vault_pda;
    
    let ix = system_instruction::transfer(
        from.key,
        to.key,
        amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[from.to_account_info(), to.to_account_info()],
    )?;
    
    let vault = &mut ctx.accounts.vault;
    vault.total_deposits = vault.total_deposits.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    let user_vault = &mut ctx.accounts.user_vault;
    if user_vault.owner == Pubkey::default() {
        user_vault.owner = from.key();
        user_vault.bump = ctx.bumps.user_vault;
    }
    user_vault.total_deposited = user_vault.total_deposited.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    user_vault.current_balance = user_vault.current_balance.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    let clock = Clock::get()?;
    user_vault.last_transaction = clock.unix_timestamp;
    
    msg!("User {} deposited {} lamports to vault", from.key(), amount);
    Ok(())
}