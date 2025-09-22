use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("1h1HCv8m2F3vL2hPCFz1dX4Srx1Lj7UQzsYShXVpXsk");

#[program]
pub mod vault_app {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, authority: Pubkey) -> Result<()> {
        instructions::initialize_vault::handler(ctx, authority)
    }

    pub fn initialize_user_vault(ctx: Context<InitializeUserVault>) -> Result<()> {
        instructions::initialize_user_vault::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, amount)
    }

    pub fn get_user_balance(ctx: Context<GetUserBalance>) -> Result<u64> {
        instructions::get_user_balance::handler(ctx)
    }

    pub fn get_vault_stats(ctx: Context<GetVaultStats>) -> Result<(u64, u64)> {
        instructions::get_vault_stats::handler(ctx)
    }

    pub fn initialize_token_vault(ctx: Context<InitializeTokenVault>) -> Result<()> {
        instructions::initialize_token_vault::handler(ctx)
    }

    pub fn wrap_sol(ctx: Context<WrapSol>, amount: u64) -> Result<()> {
        instructions::wrap_sol::handler(ctx, amount)
    }

    pub fn unwrap_sol(ctx: Context<UnwrapSol>, amount: u64) -> Result<()> {
        instructions::unwrap_sol::handler(ctx, amount)
    }

    pub fn swap_sol_to_usdc(
        ctx: Context<SwapSolToUsdc>,
        amount_in: u64,
        minimum_amount_out: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        instructions::swap_sol_to_usdc::handler(
            ctx,
            amount_in,
            minimum_amount_out,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )
    }

    pub fn user_swap_sol_to_usdc(
        ctx: Context<UserSwapSolToUsdc>,
        sol_amount: u64,
        min_usdc_out: u64,
    ) -> Result<()> {
        instructions::user_swap_sol_to_usdc::handler(ctx, sol_amount, min_usdc_out)
    }

    pub fn withdraw_usdc(ctx: Context<WithdrawUsdc>, amount: u64) -> Result<()> {
        instructions::withdraw_usdc::handler(ctx, amount)
    }

    // Admin functions - requires backend authority
    pub fn admin_swap_for_user(
        ctx: Context<AdminSwapForUser>,
        sol_amount: u64,
        min_usdc_out: u64,
    ) -> Result<()> {
        instructions::admin_swap_for_user::handler(ctx, sol_amount, min_usdc_out)
    }

    pub fn admin_wrap_sol(ctx: Context<AdminWrapSol>, amount: u64) -> Result<()> {
        instructions::admin_wrap_sol::handler(ctx, amount)
    }

    pub fn admin_withdraw_usdc(ctx: Context<AdminWithdrawUsdc>, amount: u64) -> Result<()> {
        instructions::admin_withdraw_usdc::handler(ctx, amount)
    }

    // Reverse swap functions (USDC to SOL)
    pub fn swap_usdc_to_sol(
        ctx: Context<SwapUsdcToSol>,
        amount_in: u64,
        minimum_amount_out: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        instructions::swap_usdc_to_sol::handler(
            ctx,
            amount_in,
            minimum_amount_out,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )
    }

    pub fn user_swap_usdc_to_sol(
        ctx: Context<UserSwapUsdcToSol>,
        usdc_amount: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        instructions::user_swap_usdc_to_sol::handler(ctx, usdc_amount, min_sol_out)
    }

    pub fn admin_swap_usdc_to_sol(
        ctx: Context<AdminSwapUsdcToSol>,
        usdc_amount: u64,
        min_sol_out: u64,
    ) -> Result<()> {
        instructions::admin_swap_usdc_to_sol::handler(ctx, usdc_amount, min_sol_out)
    }
}