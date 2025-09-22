use anchor_lang::prelude::*;
use crate::state::{Vault, UserVaultAccount};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        mut,
        seeds = [b"user_vault", owner.key().as_ref()],
        bump = user_vault.bump,
        has_one = owner @ ErrorCode::UnauthorizedWithdrawal
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// CHECK: The recipient of the withdrawal
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let vault_pda = &ctx.accounts.vault_pda;
    let to = &ctx.accounts.recipient;
    let user_vault = &mut ctx.accounts.user_vault;
    
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
    
    let cpi_context = anchor_lang::context::CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: vault_pda.to_account_info(),
            to: to.to_account_info(),
        },
        signer_seeds,
    );
    
    anchor_lang::system_program::transfer(cpi_context, amount)?;
    
    user_vault.current_balance = user_vault.current_balance.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    user_vault.total_withdrawn = user_vault.total_withdrawn.checked_add(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    vault.total_deposits = vault.total_deposits.checked_sub(amount)
        .ok_or(ErrorCode::MathOverflow)?;
    
    let clock = Clock::get()?;
    user_vault.last_transaction = clock.unix_timestamp;
    
    msg!("User {} withdrew {} lamports from vault", ctx.accounts.owner.key(), amount);
    Ok(())
}