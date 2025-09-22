use anchor_lang::prelude::*;
use crate::state::Vault;

#[derive(Accounts)]
pub struct GetVaultStats<'info> {
    #[account(
        seeds = [b"vault"],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,
    
    /// CHECK: This is the vault PDA that holds funds
    #[account(
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: AccountInfo<'info>,
}

pub fn handler(ctx: Context<GetVaultStats>) -> Result<(u64, u64)> {
    let vault = &ctx.accounts.vault;
    let vault_pda = &ctx.accounts.vault_pda;
    Ok((vault.total_deposits, vault_pda.get_lamports()))
}