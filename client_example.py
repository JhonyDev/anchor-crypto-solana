#!/usr/bin/env python3
"""
Vault App - Python Client Example using anchorpy
This demonstrates how to interact with the Solana vault program from Python/Django
"""

import asyncio
import json
import os
from pathlib import Path
from solana.rpc.async_api import AsyncClient
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.system_program import SYS_PROGRAM_ID
from anchorpy import Provider, Wallet, Program, Context
from anchorpy.utils.rpc import get_multiple_accounts
import base64

# Configuration
MAINNET_RPC = "https://api.mainnet-beta.solana.com"
PROGRAM_ID = "7tc3S8BCNJh2ay5AEVzPmvcL1fYZVcAtRA1Pr4utZ7Sm"
IDL_PATH = "./target/idl/vault_app.json"

class VaultClient:
    """Client for interacting with the Vault program"""
    
    def __init__(self, keypair_path: str = None):
        """
        Initialize the Vault client
        
        Args:
            keypair_path: Path to the Solana keypair JSON file
                         Defaults to ~/.config/solana/id.json
        """
        # Load keypair
        if keypair_path is None:
            keypair_path = os.path.expanduser("~/.config/solana/id.json")
        
        with open(keypair_path, 'r') as f:
            keypair_data = json.load(f)
        self.payer = Keypair.from_secret_key(bytes(keypair_data[:32]))
        self.wallet = Wallet(self.payer)
        
        # Load IDL
        with open(IDL_PATH, 'r') as f:
            self.idl = json.load(f)
        
        self.program_id = PublicKey(PROGRAM_ID)
        self.client = None
        self.provider = None
        self.program = None
    
    async def connect(self):
        """Connect to Solana mainnet-beta"""
        self.client = AsyncClient(MAINNET_RPC)
        self.provider = Provider(self.client, self.wallet)
        self.program = Program(
            idl=self.idl,
            program_id=self.program_id,
            provider=self.provider
        )
    
    async def close(self):
        """Close the RPC connection"""
        if self.client:
            await self.client.close()
    
    def get_vault_pda(self):
        """Get the vault PDA address"""
        vault_pda, bump = PublicKey.find_program_address(
            [b"vault"],
            self.program_id
        )
        return vault_pda
    
    def get_vault_funds_pda(self):
        """Get the vault funds PDA address (where SOL is stored)"""
        vault_funds_pda, bump = PublicKey.find_program_address(
            [b"vault_pda"],
            self.program_id
        )
        return vault_funds_pda
    
    async def initialize_vault(self, authority: PublicKey = None):
        """
        Initialize the vault with a specified authority
        
        Args:
            authority: The public key that will control the vault
                      Defaults to the payer's public key
        
        Returns:
            Transaction signature
        """
        if authority is None:
            authority = self.payer.public_key
        
        vault_pda = self.get_vault_pda()
        
        # Build and send transaction
        tx = await self.program.rpc["initialize_vault"](
            authority,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "payer": self.payer.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        
        print(f"Vault initialized!")
        print(f"Transaction: {tx}")
        print(f"Vault PDA: {vault_pda}")
        print(f"Authority: {authority}")
        
        return tx
    
    async def deposit(self, amount_sol: float):
        """
        Deposit SOL into the vault
        
        Args:
            amount_sol: Amount of SOL to deposit (e.g., 0.1 for 0.1 SOL)
        
        Returns:
            Transaction signature
        """
        # Convert SOL to lamports (1 SOL = 1,000,000,000 lamports)
        amount_lamports = int(amount_sol * 1_000_000_000)
        
        vault_pda = self.get_vault_pda()
        vault_funds_pda = self.get_vault_funds_pda()
        
        # Build and send transaction
        tx = await self.program.rpc["deposit"](
            amount_lamports,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "depositor": self.payer.public_key,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        
        print(f"Deposited {amount_sol} SOL to vault!")
        print(f"Transaction: {tx}")
        
        return tx
    
    async def withdraw(self, amount_sol: float, recipient: PublicKey = None):
        """
        Withdraw SOL from the vault (only authority can do this)
        
        Args:
            amount_sol: Amount of SOL to withdraw
            recipient: Recipient public key (defaults to authority)
        
        Returns:
            Transaction signature
        """
        if recipient is None:
            recipient = self.payer.public_key
        
        amount_lamports = int(amount_sol * 1_000_000_000)
        
        vault_pda = self.get_vault_pda()
        vault_funds_pda = self.get_vault_funds_pda()
        
        # Build and send transaction
        tx = await self.program.rpc["withdraw"](
            amount_lamports,
            ctx=Context(
                accounts={
                    "vault": vault_pda,
                    "vault_pda": vault_funds_pda,
                    "authority": self.payer.public_key,
                    "recipient": recipient,
                    "system_program": SYS_PROGRAM_ID,
                }
            )
        )
        
        print(f"Withdrew {amount_sol} SOL from vault!")
        print(f"Transaction: {tx}")
        
        return tx
    
    async def get_vault_balance(self):
        """
        Get the current balance of the vault
        
        Returns:
            Balance in SOL
        """
        vault_funds_pda = self.get_vault_funds_pda()
        
        # Get account info
        response = await self.client.get_account_info(vault_funds_pda)
        
        if response["result"]["value"] is None:
            print("Vault not yet initialized or has no balance")
            return 0
        
        lamports = response["result"]["value"]["lamports"]
        sol_balance = lamports / 1_000_000_000
        
        print(f"Vault balance: {sol_balance} SOL")
        return sol_balance
    
    async def get_vault_info(self):
        """
        Get vault account information (authority, total deposits, etc.)
        
        Returns:
            Vault account data
        """
        vault_pda = self.get_vault_pda()
        
        # Fetch account using anchorpy
        vault_account = await self.program.account["Vault"].fetch(vault_pda)
        
        if vault_account:
            print(f"Vault Info:")
            print(f"  Authority: {vault_account.authority}")
            print(f"  Total Deposits: {vault_account.total_deposits / 1_000_000_000} SOL")
            print(f"  Bump: {vault_account.bump}")
        
        return vault_account


# Django Integration Example
class VaultService:
    """
    Service class for Django integration
    This can be used in Django views, tasks, or management commands
    """
    
    @staticmethod
    async def deposit_to_vault(amount_sol: float, keypair_path: str = None):
        """
        Django service method to deposit to vault
        
        Usage in Django view:
        from asgiref.sync import async_to_sync
        
        def deposit_view(request):
            amount = float(request.POST.get('amount'))
            result = async_to_sync(VaultService.deposit_to_vault)(amount)
            return JsonResponse({'tx': result})
        """
        client = VaultClient(keypair_path)
        try:
            await client.connect()
            tx = await client.deposit(amount_sol)
            balance = await client.get_vault_balance()
            return {
                'transaction': str(tx),
                'new_balance': balance
            }
        finally:
            await client.close()
    
    @staticmethod
    async def get_vault_status(keypair_path: str = None):
        """
        Get vault status for Django dashboard
        """
        client = VaultClient(keypair_path)
        try:
            await client.connect()
            balance = await client.get_vault_balance()
            info = await client.get_vault_info()
            return {
                'balance': balance,
                'authority': str(info.authority) if info else None,
                'total_deposits': info.total_deposits / 1_000_000_000 if info else 0
            }
        finally:
            await client.close()


async def main():
    """
    Example usage demonstrating all functionality
    """
    print("=" * 50)
    print("Vault App - Python Client Example")
    print("=" * 50)
    
    # Initialize client
    client = VaultClient()
    
    try:
        # Connect to mainnet-beta
        print("\n1. Connecting to Solana mainnet-beta...")
        await client.connect()
        print("   Connected successfully!")
        
        # Check current vault balance
        print("\n2. Checking current vault balance...")
        balance = await client.get_vault_balance()
        
        # Initialize vault (only needed once per program)
        # Uncomment this if vault is not initialized yet:
        # print("\n3. Initializing vault...")
        # await client.initialize_vault()
        
        # Deposit 0.1 SOL
        print("\n3. Depositing 0.1 SOL to vault...")
        await client.deposit(0.1)
        
        # Check balance after deposit
        print("\n4. Checking vault balance after deposit...")
        new_balance = await client.get_vault_balance()
        
        # Get vault info
        print("\n5. Getting vault information...")
        await client.get_vault_info()
        
        # Example withdrawal (only authority can do this)
        # print("\n6. Withdrawing 0.05 SOL from vault...")
        # await client.withdraw(0.05)
        
    except Exception as e:
        print(f"\nError: {e}")
    finally:
        await client.close()
        print("\n" + "=" * 50)
        print("Example completed!")


# For Django integration, you would typically use this in views.py:
"""
# views.py example for Django

from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from asgiref.sync import async_to_sync
import json

@csrf_exempt
def deposit_view(request):
    '''Handle SOL deposits to vault'''
    if request.method == 'POST':
        data = json.loads(request.body)
        amount = float(data.get('amount', 0))
        
        # Call async function from sync Django view
        result = async_to_sync(VaultService.deposit_to_vault)(amount)
        
        return JsonResponse({
            'success': True,
            'transaction': result['transaction'],
            'new_balance': result['new_balance']
        })
    
    return JsonResponse({'error': 'Method not allowed'}, status=405)

def vault_status_view(request):
    '''Get vault status'''
    status = async_to_sync(VaultService.get_vault_status)()
    return JsonResponse(status)
"""


if __name__ == "__main__":
    # Run the example
    asyncio.run(main())