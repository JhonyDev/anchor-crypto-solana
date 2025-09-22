use anchor_lang::prelude::*;
use crate::state::UserVaultAccount;

#[derive(Accounts)]
pub struct GetUserBalance<'info> {
    #[account(
        seeds = [b"user_vault", user.key().as_ref()],
        bump = user_vault.bump
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    /// CHECK: The user whose balance we're checking
    pub user: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetUserBalance>) -> Result<u64> {
    Ok(ctx.accounts.user_vault.current_balance)
}