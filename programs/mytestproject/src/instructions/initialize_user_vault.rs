use anchor_lang::prelude::*;
use crate::state::UserVaultAccount;

#[derive(Accounts)]
pub struct InitializeUserVault<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"user_vault", owner.key().as_ref()],
        bump
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeUserVault>) -> Result<()> {
    let user_vault = &mut ctx.accounts.user_vault;
    user_vault.owner = ctx.accounts.owner.key();
    user_vault.total_deposited = 0;
    user_vault.total_withdrawn = 0;
    user_vault.current_balance = 0;
    user_vault.last_transaction = 0;
    user_vault.bump = ctx.bumps.user_vault;
    
    msg!("User vault initialized for {}", ctx.accounts.owner.key());
    Ok(())
}