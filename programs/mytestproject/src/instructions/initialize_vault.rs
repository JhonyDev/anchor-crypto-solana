use anchor_lang::prelude::*;
use crate::state::Vault;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 1,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVault>, authority: Pubkey) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.authority = authority;
    vault.total_deposits = 0;
    vault.bump = ctx.bumps.vault;
    msg!("Vault initialized with authority: {}", authority);
    Ok(())
}