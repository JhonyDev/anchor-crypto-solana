use anchor_lang::prelude::*;

pub const WSOL_MINT: Pubkey = anchor_spl::token::spl_token::native_mint::ID;

pub const USDC_MINT: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");

pub const ORCA_WHIRLPOOL_PROGRAM_ID: Pubkey = pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

pub const VAULT_SEED: &[u8] = b"vault";
pub const VAULT_PDA_SEED: &[u8] = b"vault_pda";
pub const USER_VAULT_SEED: &[u8] = b"user_vault";
pub const TOKEN_VAULT_SEED: &[u8] = b"token_vault";
pub const USER_TOKEN_ACCOUNT_SEED: &[u8] = b"user_token_account";