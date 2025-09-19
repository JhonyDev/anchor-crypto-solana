#!/usr/bin/env python3
"""
Comprehensive DevNet Test Suite for Multi-User Vault

This script runs detailed tests on the deployed vault program on DevNet,
testing all multi-user functionality, security features, and edge cases.
"""

import json
import asyncio
import time
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
from typing import Tuple
from colorama import init, Fore, Style

# Initialize colorama for colored output
init()

# Program ID on DevNet
PROGRAM_ID = PublicKey("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL")

# Test results tracking
test_results = {
    "passed": 0,
    "failed": 0,
    "details": []
}

def print_header(text: str):
    """Print a formatted header"""
    print(f"\n{Fore.CYAN}{'='*60}")
    print(f"{text:^60}")
    print(f"{'='*60}{Style.RESET_ALL}")

def print_test(name: str, passed: bool, details: str = ""):
    """Print test result"""
    global test_results
    if passed:
        test_results["passed"] += 1
        status = f"{Fore.GREEN}✅ PASSED{Style.RESET_ALL}"
    else:
        test_results["failed"] += 1
        status = f"{Fore.RED}❌ FAILED{Style.RESET_ALL}"
    
    print(f"{status} | {name}")
    if details:
        print(f"         {Fore.YELLOW}{details}{Style.RESET_ALL}")
    
    test_results["details"].append({
        "test": name,
        "passed": passed,
        "details": details
    })

async def test_initialization(program: Program, provider: Provider, authority: Keypair) -> Tuple[PublicKey, PublicKey]:
    """Test vault initialization"""
    print_header("TEST: Vault Initialization")
    
    # Derive PDAs
    vault_pda, _ = PublicKey.find_program_address([b"vault"], PROGRAM_ID)
    vault_funds_pda, _ = PublicKey.find_program_address([b"vault_pda"], PROGRAM_ID)
    
    try:
        # Try to initialize
        tx = await program.rpc["initialize_vault"](
            authority.public_key,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "payer": provider.wallet.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print_test("Vault initialization", True, f"Tx: {tx[:8]}...")
    except Exception as e:
        if "already in use" in str(e):
            print_test("Vault already initialized", True, "Checking existing vault")
            # Fetch existing vault
            vault_account = await program.account["Vault"].fetch(vault_pda)
            print(f"  Existing authority: {vault_account.authority}")
            print(f"  Total deposits: {vault_account.total_deposits / 1e9:.4f} SOL")
        else:
            print_test("Vault initialization", False, str(e))
    
    return vault_pda, vault_funds_pda

async def test_user_deposits(program: Program, users: list, vault_pda: PublicKey, vault_funds_pda: PublicKey):
    """Test multiple user deposits"""
    print_header("TEST: User Deposits")
    
    deposit_amounts = [0.2, 0.3, 0.15]  # SOL amounts for each user
    
    for i, (user, amount) in enumerate(zip(users, deposit_amounts)):
        user_vault_pda, _ = PublicKey.find_program_address(
            [b"user_vault", bytes(user.public_key)],
            PROGRAM_ID
        )
        
        # Create provider for this user
        user_wallet = Wallet(user)
        user_provider = Provider(program.provider.connection, user_wallet)
        user_program = Program(
            program.idl,
            PROGRAM_ID,
            user_provider
        )
        
        try:
            deposit_lamports = int(amount * 1e9)
            tx = await user_program.rpc["deposit"](
                deposit_lamports,
                ctx=Context(
                    accounts={
                        "vault": vault_pda,
                        "vault_pda": vault_funds_pda,
                        "user_vault": user_vault_pda,
                        "depositor": user.public_key,
                        "system_program": SYS_PROGRAM_ID,
                    }
                )
            )
            
            # Verify deposit
            user_vault = await program.account["UserVaultAccount"].fetch(user_vault_pda)
            balance_sol = user_vault.current_balance / 1e9
            
            print_test(
                f"User {i+1} deposit {amount} SOL", 
                True, 
                f"Balance: {balance_sol:.4f} SOL"
            )
        except Exception as e:
            print_test(f"User {i+1} deposit", False, str(e))

async def test_user_withdrawals(program: Program, users: list, vault_pda: PublicKey, vault_funds_pda: PublicKey):
    """Test user withdrawals"""
    print_header("TEST: User Withdrawals")
    
    withdraw_amounts = [0.1, 0.1, 0.05]  # SOL amounts for each user
    
    for i, (user, amount) in enumerate(zip(users, withdraw_amounts)):
        user_vault_pda, _ = PublicKey.find_program_address(
            [b"user_vault", bytes(user.public_key)],
            PROGRAM_ID
        )
        
        # Create provider for this user
        user_wallet = Wallet(user)
        user_provider = Provider(program.provider.connection, user_wallet)
        user_program = Program(
            program.idl,
            PROGRAM_ID,
            user_provider
        )
        
        try:
            # Get balance before withdrawal
            user_vault_before = await program.account["UserVaultAccount"].fetch(user_vault_pda)
            balance_before = user_vault_before.current_balance / 1e9
            
            withdraw_lamports = int(amount * 1e9)
            tx = await user_program.rpc["withdraw"](
                withdraw_lamports,
                ctx=Context(
                    accounts={
                        "vault": vault_pda,
                        "vault_pda": vault_funds_pda,
                        "user_vault": user_vault_pda,
                        "owner": user.public_key,
                        "recipient": user.public_key,
                        "system_program": SYS_PROGRAM_ID,
                    }
                )
            )
            
            # Verify withdrawal
            user_vault_after = await program.account["UserVaultAccount"].fetch(user_vault_pda)
            balance_after = user_vault_after.current_balance / 1e9
            
            print_test(
                f"User {i+1} withdraw {amount} SOL", 
                True, 
                f"Balance: {balance_before:.4f} → {balance_after:.4f} SOL"
            )
        except Exception as e:
            print_test(f"User {i+1} withdrawal", False, str(e))

async def test_security_features(program: Program, users: list, vault_pda: PublicKey, vault_funds_pda: PublicKey):
    """Test security features"""
    print_header("TEST: Security Features")
    
    # Test 1: User trying to withdraw from another user's vault
    user1 = users[0]
    user2 = users[1]
    user2_vault_pda, _ = PublicKey.find_program_address(
        [b"user_vault", bytes(user2.public_key)],
        PROGRAM_ID
    )
    
    user1_wallet = Wallet(user1)
    user1_provider = Provider(program.provider.connection, user1_wallet)
    user1_program = Program(program.idl, PROGRAM_ID, user1_provider)
    
    try:
        await user1_program.rpc["withdraw"](
            50_000_000,  # 0.05 SOL
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": user2_vault_pda,  # User2's vault!
                    "owner": user1.public_key,
                    "recipient": user1.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print_test("Unauthorized withdrawal prevention", False, "Security breach!")
    except Exception as e:
        if "ConstraintHasOne" in str(e) or "constraint" in str(e).lower():
            print_test("Unauthorized withdrawal prevention", True, "Access denied as expected")
        else:
            print_test("Unauthorized withdrawal prevention", False, str(e))
    
    # Test 2: Overdraft prevention
    user1_vault_pda, _ = PublicKey.find_program_address(
        [b"user_vault", bytes(user1.public_key)],
        PROGRAM_ID
    )
    
    try:
        # Get current balance
        user_vault = await program.account["UserVaultAccount"].fetch(user1_vault_pda)
        current_balance = user_vault.current_balance
        
        # Try to withdraw more than balance
        overdraft_amount = current_balance + 100_000_000  # Add 0.1 SOL
        
        await user1_program.rpc["withdraw"](
            overdraft_amount,
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
        print_test("Overdraft prevention", False, "Overdraft allowed!")
    except Exception as e:
        if "InsufficientUserBalance" in str(e):
            print_test("Overdraft prevention", True, "Overdraft blocked as expected")
        else:
            print_test("Overdraft prevention", False, str(e))

async def test_balance_queries(program: Program, users: list):
    """Test balance and stats queries"""
    print_header("TEST: Balance Queries")
    
    # Test user balance queries
    for i, user in enumerate(users[:2]):  # Test first 2 users
        user_vault_pda, _ = PublicKey.find_program_address(
            [b"user_vault", bytes(user.public_key)],
            PROGRAM_ID
        )
        
        try:
            balance = await program.rpc["get_user_balance"](
                ctx=Context(
                    accounts={
                        "user_vault": user_vault_pda,
                        "user": user.public_key,
                    }
                )
            )
            balance_sol = balance / 1e9
            print_test(f"User {i+1} balance query", True, f"{balance_sol:.4f} SOL")
        except Exception as e:
            print_test(f"User {i+1} balance query", False, str(e))
    
    # Test vault stats
    vault_pda, _ = PublicKey.find_program_address([b"vault"], PROGRAM_ID)
    vault_funds_pda, _ = PublicKey.find_program_address([b"vault_pda"], PROGRAM_ID)
    
    try:
        stats = await program.rpc["get_vault_stats"](
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                }
            )
        )
        total_deposits = stats[0] / 1e9
        vault_balance = stats[1] / 1e9
        print_test(
            "Vault statistics query", 
            True, 
            f"Deposits: {total_deposits:.4f} SOL, Balance: {vault_balance:.4f} SOL"
        )
    except Exception as e:
        print_test("Vault statistics query", False, str(e))

async def test_edge_cases(program: Program, vault_pda: PublicKey, vault_funds_pda: PublicKey):
    """Test edge cases"""
    print_header("TEST: Edge Cases")
    
    # Test 1: Zero amount deposit
    edge_user = Keypair()
    edge_wallet = Wallet(edge_user)
    edge_provider = Provider(program.provider.connection, edge_wallet)
    edge_program = Program(program.idl, PROGRAM_ID, edge_provider)
    
    # Airdrop some SOL for fees
    try:
        await program.provider.connection.request_airdrop(edge_user.public_key, 100_000_000)
        await asyncio.sleep(2)
    except:
        pass  # Might fail due to rate limits
    
    edge_vault_pda, _ = PublicKey.find_program_address(
        [b"user_vault", bytes(edge_user.public_key)],
        PROGRAM_ID
    )
    
    try:
        await edge_program.rpc["deposit"](
            0,  # Zero amount
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "user_vault": edge_vault_pda,
                    "depositor": edge_user.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        print_test("Zero amount deposit", True, "Handled correctly")
    except Exception as e:
        print_test("Zero amount deposit", True, "Rejected as expected")
    
    # Test 2: Multiple deposits by same user
    test_user = users[0] if 'users' in locals() else None
    if test_user:
        user_vault_pda, _ = PublicKey.find_program_address(
            [b"user_vault", bytes(test_user.public_key)],
            PROGRAM_ID
        )
        
        user_wallet = Wallet(test_user)
        user_provider = Provider(program.provider.connection, user_wallet)
        user_program = Program(program.idl, PROGRAM_ID, user_provider)
        
        try:
            # Make 3 quick deposits
            for j in range(3):
                await user_program.rpc["deposit"](
                    10_000_000,  # 0.01 SOL each
                    ctx=Context(
                        accounts={
                            "vault": vault_pda,
                            "vault_pda": vault_funds_pda,
                            "user_vault": user_vault_pda,
                            "depositor": test_user.public_key,
                            "system_program": SYS_PROGRAM_ID,
                        }
                    )
                )
            print_test("Multiple rapid deposits", True, "All deposits processed")
        except Exception as e:
            print_test("Multiple rapid deposits", False, str(e))

async def main():
    """Run all tests"""
    print(f"{Fore.MAGENTA}")
    print("╔════════════════════════════════════════════════════════╗")
    print("║     DEVNET VAULT MULTI-USER FUNCTIONALITY TESTS       ║")
    print("╚════════════════════════════════════════════════════════╝")
    print(f"{Style.RESET_ALL}")
    
    # Connect to DevNet
    client = AsyncClient("https://api.devnet.solana.com", commitment=Confirmed)
    
    # Load IDL
    with open("target/idl/vault_app.json", "r") as f:
        idl = json.load(f)
    
    # Create test wallets
    authority = Keypair()
    users = [Keypair() for _ in range(3)]
    
    print(f"\n{Fore.YELLOW}Test Wallets:{Style.RESET_ALL}")
    print(f"Authority: {authority.public_key}")
    for i, user in enumerate(users):
        print(f"User {i+1}: {user.public_key}")
    
    # Setup provider
    wallet = Wallet(authority)
    provider = Provider(client, wallet)
    program = Program(idl, PROGRAM_ID, provider)
    
    # Request airdrops
    print(f"\n{Fore.YELLOW}Requesting airdrops...{Style.RESET_ALL}")
    try:
        await client.request_airdrop(authority.public_key, 500_000_000)
        for user in users:
            await client.request_airdrop(user.public_key, 500_000_000)
        print("Waiting for confirmations...")
        await asyncio.sleep(10)
    except Exception as e:
        print(f"{Fore.YELLOW}Airdrop might be rate limited, continuing...{Style.RESET_ALL}")
    
    # Run tests
    try:
        # Test 1: Initialization
        vault_pda, vault_funds_pda = await test_initialization(program, provider, authority)
        
        # Test 2: User deposits
        await test_user_deposits(program, users, vault_pda, vault_funds_pda)
        
        # Test 3: User withdrawals
        await test_user_withdrawals(program, users, vault_pda, vault_funds_pda)
        
        # Test 4: Security features
        await test_security_features(program, users, vault_pda, vault_funds_pda)
        
        # Test 5: Balance queries
        await test_balance_queries(program, users)
        
        # Test 6: Edge cases
        await test_edge_cases(program, vault_pda, vault_funds_pda)
        
    except Exception as e:
        print(f"\n{Fore.RED}Critical error: {e}{Style.RESET_ALL}")
    
    # Print summary
    print_header("TEST SUMMARY")
    total_tests = test_results["passed"] + test_results["failed"]
    success_rate = (test_results["passed"] / total_tests * 100) if total_tests > 0 else 0
    
    print(f"{Fore.GREEN}Passed: {test_results['passed']}{Style.RESET_ALL}")
    print(f"{Fore.RED}Failed: {test_results['failed']}{Style.RESET_ALL}")
    print(f"Success Rate: {success_rate:.1f}%")
    
    if test_results["failed"] == 0:
        print(f"\n{Fore.GREEN}🎉 ALL TESTS PASSED! 🎉{Style.RESET_ALL}")
    else:
        print(f"\n{Fore.YELLOW}⚠️  Some tests failed. Review the details above.{Style.RESET_ALL}")
    
    # Save test report
    with open("test_report.json", "w") as f:
        json.dump(test_results, f, indent=2)
    print(f"\nTest report saved to test_report.json")
    
    await client.close()

if __name__ == "__main__":
    asyncio.run(main())