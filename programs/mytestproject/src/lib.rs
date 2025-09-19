use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL");

#[program]
pub mod vault_app {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, authority: Pubkey) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = authority;
        vault.total_deposits = 0;
        vault.bump = ctx.bumps.vault;
        msg!("Vault initialized with authority: {}", authority);
        Ok(())
    }

    pub fn initialize_user_vault(ctx: Context<InitializeUserVault>) -> Result<()> {
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

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let from = &ctx.accounts.depositor;
        let to = &ctx.accounts.vault_pda;
        
        let ix = system_instruction::transfer(
            from.key,
            to.key,
            amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[from.to_account_info(), to.to_account_info()],
        )?;
        
        let vault = &mut ctx.accounts.vault;
        vault.total_deposits = vault.total_deposits.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        let user_vault = &mut ctx.accounts.user_vault;
        if user_vault.owner == Pubkey::default() {
            user_vault.owner = from.key();
            user_vault.bump = ctx.bumps.user_vault;
        }
        user_vault.total_deposited = user_vault.total_deposited.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        user_vault.current_balance = user_vault.current_balance.checked_add(amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        let clock = Clock::get()?;
        user_vault.last_transaction = clock.unix_timestamp;
        
        msg!("User {} deposited {} lamports to vault", from.key(), amount);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
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

    pub fn get_user_balance(ctx: Context<GetUserBalance>) -> Result<u64> {
        Ok(ctx.accounts.user_vault.current_balance)
    }

    pub fn get_vault_stats(ctx: Context<GetVaultStats>) -> Result<(u64, u64)> {
        let vault = &ctx.accounts.vault;
        let vault_pda = &ctx.accounts.vault_pda;
        Ok((vault.total_deposits, vault_pda.get_lamports()))
    }
}

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

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        init_if_needed,
        payer = depositor,
        space = 8 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"user_vault", depositor.key().as_ref()],
        bump
    )]
    pub user_vault: Account<'info, UserVaultAccount>,
    
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

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
}