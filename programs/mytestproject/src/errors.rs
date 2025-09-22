use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized withdrawal attempt")]
    UnauthorizedWithdrawal,
    #[msg("Insufficient funds in vault")]
    InsufficientFunds,
    #[msg("Insufficient balance in user account")]
    InsufficientUserBalance,
    #[msg("Math overflow occurred")]
    MathOverflow,
    #[msg("Insufficient SOL balance for swap")]
    InsufficientSOLBalance,
    #[msg("Insufficient wSOL balance for swap")]
    InsufficientWSOLBalance,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Invalid pool configuration")]
    InvalidPoolConfiguration,
    #[msg("Swap failed")]
    SwapFailed,
    #[msg("Failed to wrap SOL")]
    WrapFailed,
    #[msg("Failed to unwrap SOL")]
    UnwrapFailed,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Token account not initialized")]
    TokenAccountNotInitialized,
}