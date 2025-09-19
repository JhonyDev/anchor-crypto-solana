#!/usr/bin/env python3
"""
Multi-User Vault Client Example

This script demonstrates the multi-user functionality of the modified vault program.
It shows how multiple users can independently deposit and withdraw their funds.
"""

import json
import asyncio
from pathlib import Path
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.system_program import SYS_PROGRAM_ID
from solana.transaction import Transaction
from anchorpy import Program, Provider, Wallet, Context
from anchorpy.error import ProgramError
import base58

# Program ID
PROGRAM_ID = PublicKey("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL")

async def main():
    # Connect to devnet
    client = AsyncClient("https://api.devnet.solana.com", commitment=Confirmed)
    
    # Load the IDL
    with open("target/idl/vault_app.json", "r") as f:
        idl = json.load(f)
    
    # Create wallets
    authority = Keypair()
    user1 = Keypair()
    user2 = Keypair()
    
    print("=== Multi-User Vault Demo ===\n")
    print(f"Authority: {authority.public_key}")
    print(f"User 1: {user1.public_key}")
    print(f"User 2: {user2.public_key}\n")
    
    # Request airdrops
    print("Requesting airdrops...")
    await client.request_airdrop(authority.public_key, 1_000_000_000)
    await client.request_airdrop(user1.public_key, 2_000_000_000)
    await client.request_airdrop(user2.public_key, 2_000_000_000)
    await asyncio.sleep(5)  # Wait for confirmations
    
    # Create providers for each user
    authority_wallet = Wallet(authority)
    authority_provider = Provider(client, authority_wallet)
    
    user1_wallet = Wallet(user1)
    user1_provider = Provider(client, user1_wallet)
    
    user2_wallet = Wallet(user2)
    user2_provider = Provider(client, user2_wallet)
    
    # Create program instances
    authority_program = Program(idl, PROGRAM_ID, authority_provider)
    user1_program = Program(idl, PROGRAM_ID, user1_provider)
    user2_program = Program(idl, PROGRAM_ID, user2_provider)
    
    # Derive PDAs
    vault_pda, vault_bump = PublicKey.find_program_address(
        [b"vault"],
        PROGRAM_ID
    )
    
    vault_funds_pda, vault_funds_bump = PublicKey.find_program_address(
        [b"vault_pda"],
        PROGRAM_ID
    )
    
    user1_vault_pda, user1_vault_bump = PublicKey.find_program_address(
        [b"user_vault", bytes(user1.public_key)],
        PROGRAM_ID
    )
    
    user2_vault_pda, user2_vault_bump = PublicKey.find_program_address(
        [b"user_vault", bytes(user2.public_key)],
        PROGRAM_ID
    )
    
    print(f"Vault PDA: {vault_pda}")
    print(f"Vault Funds PDA: {vault_funds_pda}")
    print(f"User1 Vault PDA: {user1_vault_pda}")
    print(f"User2 Vault PDA: {user2_vault_pda}\n")
    
    # Initialize the vault
    print("1. Initializing vault...")
    try:
        tx = await authority_program.rpc["initialize_vault"](
            authority.public_key,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "payer": authority.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print(f"   ✓ Vault initialized: {tx}\n")
    except Exception as e:
        print(f"   ✗ Error initializing vault: {e}\n")
    
    # User 1 deposits
    print("2. User 1 depositing 0.5 SOL...")
    deposit_amount = 500_000_000  # 0.5 SOL
    try:
        tx = await user1_program.rpc["deposit"](
            deposit_amount,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": user1_vault_pda,
                    "depositor": user1.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print(f"   ✓ User 1 deposited: {tx}")
        
        # Check balance
        user1_vault = await user1_program.account["UserVaultAccount"].fetch(user1_vault_pda)
        print(f"   User 1 balance: {user1_vault.current_balance / 1e9:.2f} SOL\n")
    except Exception as e:
        print(f"   ✗ Error depositing: {e}\n")
    
    # User 2 deposits
    print("3. User 2 depositing 0.8 SOL...")
    deposit_amount = 800_000_000  # 0.8 SOL
    try:
        tx = await user2_program.rpc["deposit"](
            deposit_amount,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": user2_vault_pda,
                    "depositor": user2.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print(f"   ✓ User 2 deposited: {tx}")
        
        # Check balance
        user2_vault = await user2_program.account["UserVaultAccount"].fetch(user2_vault_pda)
        print(f"   User 2 balance: {user2_vault.current_balance / 1e9:.2f} SOL\n")
    except Exception as e:
        print(f"   ✗ Error depositing: {e}\n")
    
    # Get vault stats
    print("4. Getting vault statistics...")
    try:
        vault_account = await authority_program.account["Vault"].fetch(vault_pda)
        vault_balance = await client.get_balance(vault_funds_pda)
        print(f"   Total deposits tracked: {vault_account.total_deposits / 1e9:.2f} SOL")
        print(f"   Actual vault balance: {vault_balance.value / 1e9:.2f} SOL\n")
    except Exception as e:
        print(f"   ✗ Error getting stats: {e}\n")
    
    # User 1 withdraws
    print("5. User 1 withdrawing 0.2 SOL...")
    withdraw_amount = 200_000_000  # 0.2 SOL
    try:
        tx = await user1_program.rpc["withdraw"](
            withdraw_amount,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": user1_vault_pda,
                    "owner": user1.public_key,
                    "recipient": user1.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print(f"   ✓ User 1 withdrew: {tx}")
        
        # Check updated balance
        user1_vault = await user1_program.account["UserVaultAccount"].fetch(user1_vault_pda)
        print(f"   User 1 remaining balance: {user1_vault.current_balance / 1e9:.2f} SOL")
        print(f"   User 1 total withdrawn: {user1_vault.total_withdrawn / 1e9:.2f} SOL\n")
    except Exception as e:
        print(f"   ✗ Error withdrawing: {e}\n")
    
    # Try unauthorized withdrawal (User 1 trying to withdraw from User 2's account)
    print("6. Testing security: User 1 trying to withdraw from User 2's account...")
    try:
        tx = await user1_program.rpc["withdraw"](
            100_000_000,  # 0.1 SOL
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": user2_vault_pda,  # User 2's vault!
                    "owner": user1.public_key,
                    "recipient": user1.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print(f"   ✗ SECURITY ISSUE: Unauthorized withdrawal succeeded!")
    except ProgramError as e:
        print(f"   ✓ Security check passed: {e}")
    except Exception as e:
        print(f"   ✓ Security check passed: Unauthorized withdrawal blocked\n")
    
    # Final balances
    print("7. Final account balances:")
    try:
        user1_vault = await user1_program.account["UserVaultAccount"].fetch(user1_vault_pda)
        user2_vault = await user2_program.account["UserVaultAccount"].fetch(user2_vault_pda)
        vault_account = await authority_program.account["Vault"].fetch(vault_pda)
        vault_balance = await client.get_balance(vault_funds_pda)
        
        print(f"   User 1:")
        print(f"     - Current balance: {user1_vault.current_balance / 1e9:.2f} SOL")
        print(f"     - Total deposited: {user1_vault.total_deposited / 1e9:.2f} SOL")
        print(f"     - Total withdrawn: {user1_vault.total_withdrawn / 1e9:.2f} SOL")
        
        print(f"   User 2:")
        print(f"     - Current balance: {user2_vault.current_balance / 1e9:.2f} SOL")
        print(f"     - Total deposited: {user2_vault.total_deposited / 1e9:.2f} SOL")
        print(f"     - Total withdrawn: {user2_vault.total_withdrawn / 1e9:.2f} SOL")
        
        print(f"   Vault:")
        print(f"     - Total deposits tracked: {vault_account.total_deposits / 1e9:.2f} SOL")
        print(f"     - Actual balance: {vault_balance.value / 1e9:.2f} SOL")
        
    except Exception as e:
        print(f"   ✗ Error getting final balances: {e}")
    
    await client.close()
    print("\n=== Demo Complete ===")

if __name__ == "__main__":
    asyncio.run(main())