use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub total_deposits: u64,
    pub bump: u8,
}

#[account]
pub struct UserVaultAccount {
    pub owner: Pubkey,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub current_balance: u64,
    pub last_transaction: i64,
    pub bump: u8,
}

#[account]
pub struct TokenVault {
    pub authority: Pubkey,
    pub vault_wsol_ata: Pubkey,
    pub vault_usdc_ata: Pubkey,
    pub total_wsol: u64,
    pub total_usdc: u64,
    pub bump: u8,
}

#[account]
pub struct UserTokenAccount {
    pub owner: Pubkey,
    pub wsol_balance: u64,
    pub usdc_balance: u64,
    pub last_swap_timestamp: i64,
    pub total_swapped: u64,
    pub bump: u8,
}

#[account]
pub struct SwapState {
    pub total_sol_swapped: u64,
    pub total_usdc_received: u64,
    pub last_swap_price: u64,
    pub swap_count: u64,
}