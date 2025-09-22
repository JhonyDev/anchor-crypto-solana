use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::TokenVault;
use crate::constants::{WSOL_MINT, USDC_MINT, TOKEN_VAULT_SEED};

#[derive(Accounts)]
pub struct InitializeTokenVault<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 32 + 8 + 8 + 1,
        seeds = [TOKEN_VAULT_SEED],
        bump
    )]
    pub token_vault: Account<'info, TokenVault>,
    
    /// CHECK: This is the vault PDA that will own the token accounts
    #[account(
        seeds = [b"vault_pda"],
        bump
    )]
    pub vault_pda: AccountInfo<'info>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = wsol_mint,
        associated_token::authority = vault_pda,
    )]
    pub vault_wsol_ata: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = payer,
        associated_token::mint = usdc_mint,
        associated_token::authority = vault_pda,
    )]
    pub vault_usdc_ata: Account<'info, TokenAccount>,
    
    #[account(address = WSOL_MINT)]
    pub wsol_mint: Account<'info, Mint>,
    
    #[account(address = USDC_MINT)]
    pub usdc_mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeTokenVault>) -> Result<()> {
    let token_vault = &mut ctx.accounts.token_vault;
    
    token_vault.authority = ctx.accounts.authority.key();
    token_vault.vault_wsol_ata = ctx.accounts.vault_wsol_ata.key();
    token_vault.vault_usdc_ata = ctx.accounts.vault_usdc_ata.key();
    token_vault.total_wsol = 0;
    token_vault.total_usdc = 0;
    token_vault.bump = ctx.bumps.token_vault;
    
    msg!("Token vault initialized with wSOL ATA: {} and USDC ATA: {}", 
         token_vault.vault_wsol_ata, 
         token_vault.vault_usdc_ata);
    
    Ok(())
}