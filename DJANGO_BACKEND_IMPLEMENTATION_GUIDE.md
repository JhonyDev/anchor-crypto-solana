# Django Backend Implementation Guide for SOL to USDC Swap

## Overview
This guide details how to integrate the new SOL to USDC swap functionality into your Django backend, including API endpoints, transaction building, and interaction with the updated Anchor program.

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Program Constants](#program-constants)
4. [Models Update](#models-update)
5. [Serializers](#serializers)
6. [Service Layer](#service-layer)
7. [API Views](#api-views)
8. [URL Configuration](#url-configuration)
9. [Celery Tasks](#celery-tasks)
10. [Testing](#testing)

## Prerequisites

### Required Python Packages
```bash
pip install solana==0.30.2
pip install anchorpy==0.18.0
pip install base58==2.1.1
pip install solders==0.18.1
pip install python-dotenv==1.0.0
```

### Environment Variables
```env
# .env file
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
WSOL_MINT=So11111111111111111111111111111111111112
USDC_MINT_DEVNET=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
ORCA_WHIRLPOOL_PROGRAM=whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc

# Orca Pool Configuration (DevNet)
ORCA_SOL_USDC_POOL=<POOL_ADDRESS>  # You'll need to find/create this
ORCA_TICK_SPACING=64
```

## Program Constants

Create a constants file for your Django app:

```python
# vault/constants.py
from solders.pubkey import Pubkey
from solana.constants import LAMPORTS_PER_SOL
import os
from dotenv import load_dotenv

load_dotenv()

# Program IDs
VAULT_PROGRAM_ID = Pubkey.from_string(os.getenv("PROGRAM_ID"))
ORCA_WHIRLPOOL_PROGRAM = Pubkey.from_string(os.getenv("ORCA_WHIRLPOOL_PROGRAM"))

# Token Mints
WSOL_MINT = Pubkey.from_string(os.getenv("WSOL_MINT"))
USDC_MINT = Pubkey.from_string(os.getenv("USDC_MINT_DEVNET"))

# Seeds for PDAs
VAULT_SEED = b"vault"
VAULT_PDA_SEED = b"vault_pda"
USER_VAULT_SEED = b"user_vault"
TOKEN_VAULT_SEED = b"token_vault"
USER_TOKEN_ACCOUNT_SEED = b"user_token_account"
SWAP_STATE_SEED = b"swap_state"

# Swap Configuration
DEFAULT_SLIPPAGE_BPS = 50  # 0.5%
MAX_SLIPPAGE_BPS = 500     # 5%

# DevNet Faucets
DEVNET_USDC_FAUCET = "https://everlastingsong.github.io/nebula/"
```

## Models Update

Add new models to track swap operations:

```python
# vault/models.py
from django.db import models
from django.contrib.auth.models import User
from django.utils import timezone
from decimal import Decimal

class UserVault(models.Model):
    """Existing vault model - update if needed"""
    user = models.OneToOneField(User, on_delete=models.CASCADE)
    pubkey = models.CharField(max_length=44, unique=True)
    sol_balance = models.DecimalField(max_digits=20, decimal_places=9, default=0)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

class TokenVault(models.Model):
    """Token vault for managing wSOL and USDC"""
    user_vault = models.OneToOneField(UserVault, on_delete=models.CASCADE, related_name='token_vault')
    wsol_ata = models.CharField(max_length=44, blank=True)
    usdc_ata = models.CharField(max_length=44, blank=True)
    wsol_balance = models.DecimalField(max_digits=20, decimal_places=9, default=0)
    usdc_balance = models.DecimalField(max_digits=20, decimal_places=6, default=0)
    initialized = models.BooleanField(default=False)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

class SwapTransaction(models.Model):
    """Track swap transactions"""
    STATUS_CHOICES = [
        ('pending', 'Pending'),
        ('processing', 'Processing'),
        ('completed', 'Completed'),
        ('failed', 'Failed'),
    ]
    
    user_vault = models.ForeignKey(UserVault, on_delete=models.CASCADE, related_name='swaps')
    transaction_hash = models.CharField(max_length=128, unique=True, null=True)
    sol_amount = models.DecimalField(max_digits=20, decimal_places=9)
    usdc_amount = models.DecimalField(max_digits=20, decimal_places=6)
    minimum_usdc_amount = models.DecimalField(max_digits=20, decimal_places=6)
    actual_usdc_received = models.DecimalField(max_digits=20, decimal_places=6, null=True)
    price_per_sol = models.DecimalField(max_digits=20, decimal_places=6, null=True)
    slippage_bps = models.IntegerField(default=50)
    status = models.CharField(max_length=20, choices=STATUS_CHOICES, default='pending')
    error_message = models.TextField(blank=True)
    created_at = models.DateTimeField(auto_now_add=True)
    completed_at = models.DateTimeField(null=True)

class OrcaPoolInfo(models.Model):
    """Cache Orca pool information"""
    pool_address = models.CharField(max_length=44, unique=True)
    token_a_mint = models.CharField(max_length=44)  # wSOL
    token_b_mint = models.CharField(max_length=44)  # USDC
    token_vault_a = models.CharField(max_length=44)
    token_vault_b = models.CharField(max_length=44)
    tick_spacing = models.IntegerField()
    current_price = models.DecimalField(max_digits=20, decimal_places=6)
    liquidity = models.DecimalField(max_digits=30, decimal_places=0)
    last_updated = models.DateTimeField(auto_now=True)
```

## Serializers

Create serializers for API endpoints:

```python
# vault/serializers.py
from rest_framework import serializers
from .models import UserVault, TokenVault, SwapTransaction

class TokenVaultSerializer(serializers.ModelSerializer):
    class Meta:
        model = TokenVault
        fields = ['wsol_ata', 'usdc_ata', 'wsol_balance', 'usdc_balance', 'initialized']
        read_only_fields = fields

class SwapEstimateSerializer(serializers.Serializer):
    sol_amount = serializers.DecimalField(max_digits=20, decimal_places=9)
    slippage_bps = serializers.IntegerField(default=50, min_value=0, max_value=500)

class SwapRequestSerializer(serializers.Serializer):
    sol_amount = serializers.DecimalField(max_digits=20, decimal_places=9, min_value=0.001)
    minimum_usdc_amount = serializers.DecimalField(max_digits=20, decimal_places=6, required=False)
    slippage_bps = serializers.IntegerField(default=50, min_value=0, max_value=500)

class SwapTransactionSerializer(serializers.ModelSerializer):
    class Meta:
        model = SwapTransaction
        fields = '__all__'
        read_only_fields = ['transaction_hash', 'actual_usdc_received', 'price_per_sol', 
                          'status', 'error_message', 'completed_at']

class WithdrawUSDCSerializer(serializers.Serializer):
    amount = serializers.DecimalField(max_digits=20, decimal_places=6, min_value=0.01)
    recipient_address = serializers.CharField(max_length=44, required=False)
```

## Service Layer

Create service classes for Solana interactions:

```python
# vault/services/solana_service.py
import asyncio
from typing import Optional, Tuple, Dict, Any
from decimal import Decimal
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed
from solana.transaction import Transaction
from solders.pubkey import Pubkey
from solders.keypair import Keypair
from solders.instruction import Instruction, AccountMeta
from solders.system_program import ID as SYSTEM_PROGRAM_ID
from solders.sysvar import RENT
from spl.token.constants import TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
from spl.token.async_client import AsyncToken
from spl.token._layouts import MINT_LAYOUT
from anchorpy import Provider, Program, Context
from anchorpy.error import ProgramError
import base58
import struct
from ..constants import *

class VaultProgramService:
    """Service for interacting with the vault program"""
    
    def __init__(self, rpc_url: str = None):
        self.rpc_url = rpc_url or os.getenv("SOLANA_RPC_URL")
        self.client = AsyncClient(self.rpc_url)
        
    async def get_pda_addresses(self, user_pubkey: Pubkey) -> Dict[str, Tuple[Pubkey, int]]:
        """Get all PDA addresses for a user"""
        pdas = {}
        
        # Vault PDA
        pdas['vault'] = Pubkey.find_program_address(
            [VAULT_SEED],
            VAULT_PROGRAM_ID
        )
        
        # Vault PDA (holds funds)
        pdas['vault_pda'] = Pubkey.find_program_address(
            [VAULT_PDA_SEED],
            VAULT_PROGRAM_ID
        )
        
        # User Vault
        pdas['user_vault'] = Pubkey.find_program_address(
            [USER_VAULT_SEED, bytes(user_pubkey)],
            VAULT_PROGRAM_ID
        )
        
        # Token Vault
        pdas['token_vault'] = Pubkey.find_program_address(
            [TOKEN_VAULT_SEED],
            VAULT_PROGRAM_ID
        )
        
        # User Token Account
        pdas['user_token_account'] = Pubkey.find_program_address(
            [USER_TOKEN_ACCOUNT_SEED, bytes(user_pubkey)],
            VAULT_PROGRAM_ID
        )
        
        # Swap State
        pdas['swap_state'] = Pubkey.find_program_address(
            [SWAP_STATE_SEED],
            VAULT_PROGRAM_ID
        )
        
        return pdas
    
    async def get_associated_token_address(self, mint: Pubkey, owner: Pubkey) -> Pubkey:
        """Get associated token address"""
        return Pubkey.find_program_address(
            [
                bytes(owner),
                bytes(TOKEN_PROGRAM_ID),
                bytes(mint)
            ],
            ASSOCIATED_TOKEN_PROGRAM_ID
        )[0]
    
    async def initialize_token_vault(self, user_pubkey: Pubkey, payer_keypair: Keypair) -> str:
        """Initialize token vault for the user"""
        pdas = await self.get_pda_addresses(user_pubkey)
        
        # Get ATAs for vault PDA
        vault_wsol_ata = await self.get_associated_token_address(WSOL_MINT, pdas['vault_pda'][0])
        vault_usdc_ata = await self.get_associated_token_address(USDC_MINT, pdas['vault_pda'][0])
        
        # Build instruction
        accounts = [
            AccountMeta(pdas['token_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['vault_pda'][0], is_signer=False, is_writable=False),
            AccountMeta(vault_wsol_ata, is_signer=False, is_writable=True),
            AccountMeta(vault_usdc_ata, is_signer=False, is_writable=True),
            AccountMeta(WSOL_MINT, is_signer=False, is_writable=False),
            AccountMeta(USDC_MINT, is_signer=False, is_writable=False),
            AccountMeta(payer_keypair.pubkey(), is_signer=True, is_writable=True),
            AccountMeta(payer_keypair.pubkey(), is_signer=True, is_writable=True),
            AccountMeta(TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
        ]
        
        # Instruction discriminator for 'initialize_token_vault'
        discriminator = self._get_discriminator("initialize_token_vault")
        
        ix = Instruction(
            program_id=VAULT_PROGRAM_ID,
            accounts=accounts,
            data=discriminator
        )
        
        # Send transaction
        tx = Transaction().add(ix)
        result = await self.client.send_transaction(tx, payer_keypair)
        await self.client.confirm_transaction(result['result'])
        
        return result['result']
    
    async def wrap_sol(self, user_pubkey: Pubkey, amount: int, user_keypair: Keypair) -> str:
        """Wrap SOL to wSOL"""
        pdas = await self.get_pda_addresses(user_pubkey)
        vault_wsol_ata = await self.get_associated_token_address(WSOL_MINT, pdas['vault_pda'][0])
        
        accounts = [
            AccountMeta(pdas['token_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['vault_pda'][0], is_signer=False, is_writable=True),
            AccountMeta(vault_wsol_ata, is_signer=False, is_writable=True),
            AccountMeta(WSOL_MINT, is_signer=False, is_writable=False),
            AccountMeta(pdas['user_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=False),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=False),
            AccountMeta(TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
        ]
        
        discriminator = self._get_discriminator("wrap_sol")
        data = discriminator + struct.pack("<Q", amount)
        
        ix = Instruction(
            program_id=VAULT_PROGRAM_ID,
            accounts=accounts,
            data=data
        )
        
        tx = Transaction().add(ix)
        result = await self.client.send_transaction(tx, user_keypair)
        await self.client.confirm_transaction(result['result'])
        
        return result['result']
    
    async def user_swap_sol_to_usdc(
        self, 
        user_pubkey: Pubkey,
        sol_amount: int,
        min_usdc_out: int,
        user_keypair: Keypair
    ) -> str:
        """Execute user swap from SOL to USDC"""
        pdas = await self.get_pda_addresses(user_pubkey)
        vault_wsol_ata = await self.get_associated_token_address(WSOL_MINT, pdas['vault_pda'][0])
        vault_usdc_ata = await self.get_associated_token_address(USDC_MINT, pdas['vault_pda'][0])
        
        accounts = [
            AccountMeta(pdas['user_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['user_token_account'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['token_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(vault_wsol_ata, is_signer=False, is_writable=True),
            AccountMeta(vault_usdc_ata, is_signer=False, is_writable=True),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=True),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=False),
            AccountMeta(SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
        ]
        
        discriminator = self._get_discriminator("user_swap_sol_to_usdc")
        data = discriminator + struct.pack("<QQ", sol_amount, min_usdc_out)
        
        ix = Instruction(
            program_id=VAULT_PROGRAM_ID,
            accounts=accounts,
            data=data
        )
        
        tx = Transaction().add(ix)
        result = await self.client.send_transaction(tx, user_keypair)
        await self.client.confirm_transaction(result['result'])
        
        return result['result']
    
    async def withdraw_usdc(
        self,
        user_pubkey: Pubkey,
        amount: int,
        user_keypair: Keypair
    ) -> str:
        """Withdraw USDC from vault to user wallet"""
        pdas = await self.get_pda_addresses(user_pubkey)
        vault_usdc_ata = await self.get_associated_token_address(USDC_MINT, pdas['vault_pda'][0])
        user_usdc_ata = await self.get_associated_token_address(USDC_MINT, user_pubkey)
        
        accounts = [
            AccountMeta(pdas['user_token_account'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['token_vault'][0], is_signer=False, is_writable=True),
            AccountMeta(pdas['vault_pda'][0], is_signer=False, is_writable=False),
            AccountMeta(vault_usdc_ata, is_signer=False, is_writable=True),
            AccountMeta(user_usdc_ata, is_signer=False, is_writable=True),
            AccountMeta(USDC_MINT, is_signer=False, is_writable=False),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=True),
            AccountMeta(user_keypair.pubkey(), is_signer=True, is_writable=False),
            AccountMeta(TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
            AccountMeta(SYSTEM_PROGRAM_ID, is_signer=False, is_writable=False),
        ]
        
        discriminator = self._get_discriminator("withdraw_usdc")
        data = discriminator + struct.pack("<Q", amount)
        
        ix = Instruction(
            program_id=VAULT_PROGRAM_ID,
            accounts=accounts,
            data=data
        )
        
        tx = Transaction().add(ix)
        result = await self.client.send_transaction(tx, user_keypair)
        await self.client.confirm_transaction(result['result'])
        
        return result['result']
    
    def _get_discriminator(self, instruction_name: str) -> bytes:
        """Generate instruction discriminator"""
        import hashlib
        preimage = f"global:{instruction_name}"
        return hashlib.sha256(preimage.encode()).digest()[:8]
    
    async def close(self):
        """Close the async client"""
        await self.client.close()


class OrcaService:
    """Service for interacting with Orca DEX"""
    
    def __init__(self):
        self.client = AsyncClient(os.getenv("SOLANA_RPC_URL"))
        
    async def get_pool_price(self, pool_address: Pubkey) -> Decimal:
        """Get current pool price"""
        # This is a simplified version - you'll need to implement actual Orca pool reading
        # For now, return a mock price
        return Decimal("40.50")  # Mock: 1 SOL = 40.50 USDC
    
    async def estimate_swap_output(
        self,
        amount_in: Decimal,
        slippage_bps: int = 50
    ) -> Dict[str, Decimal]:
        """Estimate swap output with slippage"""
        price = await self.get_pool_price(Pubkey.from_string(os.getenv("ORCA_SOL_USDC_POOL", "")))
        
        estimated_output = amount_in * price
        slippage_factor = Decimal(1) - (Decimal(slippage_bps) / Decimal(10000))
        minimum_output = estimated_output * slippage_factor
        
        return {
            'estimated_output': estimated_output,
            'minimum_output': minimum_output,
            'price_per_sol': price,
            'price_impact': Decimal("0.01"),  # Mock 0.01% price impact
        }
    
    async def close(self):
        await self.client.close()
```

## API Views

Create API views for the swap functionality:

```python
# vault/views.py
from rest_framework import status
from rest_framework.decorators import api_view, permission_classes
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from django.db import transaction
from decimal import Decimal
import asyncio
from solders.pubkey import Pubkey
from solders.keypair import Keypair
import base58

from .models import UserVault, TokenVault, SwapTransaction
from .serializers import (
    TokenVaultSerializer,
    SwapEstimateSerializer,
    SwapRequestSerializer,
    SwapTransactionSerializer,
    WithdrawUSDCSerializer
)
from .services.solana_service import VaultProgramService, OrcaService
from .tasks import process_swap_transaction

@api_view(['POST'])
@permission_classes([IsAuthenticated])
def initialize_token_vault(request):
    """Initialize token vault for the user"""
    try:
        user_vault = request.user.uservault
        token_vault, created = TokenVault.objects.get_or_create(user_vault=user_vault)
        
        if token_vault.initialized:
            return Response({
                'message': 'Token vault already initialized',
                'data': TokenVaultSerializer(token_vault).data
            })
        
        # Get user's keypair (you should have this stored securely)
        # For demo, using a dummy keypair
        user_pubkey = Pubkey.from_string(user_vault.pubkey)
        payer_keypair = Keypair()  # In production, use actual payer
        
        # Initialize on-chain
        service = VaultProgramService()
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            tx_hash = loop.run_until_complete(
                service.initialize_token_vault(user_pubkey, payer_keypair)
            )
            
            # Update model
            pdas = loop.run_until_complete(service.get_pda_addresses(user_pubkey))
            vault_pda = pdas['vault_pda'][0]
            
            wsol_ata = loop.run_until_complete(
                service.get_associated_token_address(
                    Pubkey.from_string(os.getenv("WSOL_MINT")),
                    vault_pda
                )
            )
            usdc_ata = loop.run_until_complete(
                service.get_associated_token_address(
                    Pubkey.from_string(os.getenv("USDC_MINT_DEVNET")),
                    vault_pda
                )
            )
            
            token_vault.wsol_ata = str(wsol_ata)
            token_vault.usdc_ata = str(usdc_ata)
            token_vault.initialized = True
            token_vault.save()
            
            return Response({
                'message': 'Token vault initialized successfully',
                'transaction': tx_hash,
                'data': TokenVaultSerializer(token_vault).data
            }, status=status.HTTP_201_CREATED)
            
        finally:
            loop.run_until_complete(service.close())
            loop.close()
            
    except Exception as e:
        return Response({
            'error': str(e)
        }, status=status.HTTP_400_BAD_REQUEST)

@api_view(['POST'])
@permission_classes([IsAuthenticated])
def estimate_swap(request):
    """Estimate swap output"""
    serializer = SwapEstimateSerializer(data=request.data)
    if not serializer.is_valid():
        return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)
    
    sol_amount = serializer.validated_data['sol_amount']
    slippage_bps = serializer.validated_data['slippage_bps']
    
    service = OrcaService()
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        estimate = loop.run_until_complete(
            service.estimate_swap_output(sol_amount, slippage_bps)
        )
        
        return Response({
            'sol_amount': str(sol_amount),
            'estimated_usdc': str(estimate['estimated_output']),
            'minimum_usdc': str(estimate['minimum_output']),
            'price_per_sol': str(estimate['price_per_sol']),
            'price_impact': str(estimate['price_impact']),
            'slippage_bps': slippage_bps
        })
        
    finally:
        loop.run_until_complete(service.close())
        loop.close()

@api_view(['POST'])
@permission_classes([IsAuthenticated])
def swap_sol_to_usdc(request):
    """Execute SOL to USDC swap"""
    serializer = SwapRequestSerializer(data=request.data)
    if not serializer.is_valid():
        return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)
    
    try:
        user_vault = request.user.uservault
        token_vault = user_vault.token_vault
        
        if not token_vault.initialized:
            return Response({
                'error': 'Token vault not initialized'
            }, status=status.HTTP_400_BAD_REQUEST)
        
        sol_amount = serializer.validated_data['sol_amount']
        slippage_bps = serializer.validated_data['slippage_bps']
        
        # Check balance
        if user_vault.sol_balance < sol_amount:
            return Response({
                'error': 'Insufficient SOL balance'
            }, status=status.HTTP_400_BAD_REQUEST)
        
        # Get estimate
        orca_service = OrcaService()
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            estimate = loop.run_until_complete(
                orca_service.estimate_swap_output(sol_amount, slippage_bps)
            )
            
            minimum_usdc = serializer.validated_data.get('minimum_usdc_amount') or estimate['minimum_output']
            
            # Create swap transaction record
            swap_tx = SwapTransaction.objects.create(
                user_vault=user_vault,
                sol_amount=sol_amount,
                usdc_amount=estimate['estimated_output'],
                minimum_usdc_amount=minimum_usdc,
                slippage_bps=slippage_bps,
                status='pending'
            )
            
            # Queue async processing
            process_swap_transaction.delay(swap_tx.id)
            
            return Response({
                'message': 'Swap initiated',
                'swap_id': swap_tx.id,
                'estimated_usdc': str(estimate['estimated_output']),
                'minimum_usdc': str(minimum_usdc)
            }, status=status.HTTP_202_ACCEPTED)
            
        finally:
            loop.run_until_complete(orca_service.close())
            loop.close()
            
    except Exception as e:
        return Response({
            'error': str(e)
        }, status=status.HTTP_400_BAD_REQUEST)

@api_view(['POST'])
@permission_classes([IsAuthenticated])
def withdraw_usdc(request):
    """Withdraw USDC from vault"""
    serializer = WithdrawUSDCSerializer(data=request.data)
    if not serializer.is_valid():
        return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)
    
    try:
        user_vault = request.user.uservault
        token_vault = user_vault.token_vault
        
        amount = serializer.validated_data['amount']
        
        if token_vault.usdc_balance < amount:
            return Response({
                'error': 'Insufficient USDC balance'
            }, status=status.HTTP_400_BAD_REQUEST)
        
        # Get user keypair (in production, handle this securely)
        user_pubkey = Pubkey.from_string(user_vault.pubkey)
        user_keypair = Keypair()  # Replace with actual user keypair
        
        # Convert USDC amount to smallest unit (6 decimals)
        amount_smallest = int(amount * Decimal('1000000'))
        
        # Execute withdrawal
        service = VaultProgramService()
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            tx_hash = loop.run_until_complete(
                service.withdraw_usdc(user_pubkey, amount_smallest, user_keypair)
            )
            
            # Update balance
            token_vault.usdc_balance -= amount
            token_vault.save()
            
            return Response({
                'message': 'USDC withdrawal successful',
                'transaction': tx_hash,
                'amount': str(amount),
                'remaining_balance': str(token_vault.usdc_balance)
            })
            
        finally:
            loop.run_until_complete(service.close())
            loop.close()
            
    except Exception as e:
        return Response({
            'error': str(e)
        }, status=status.HTTP_400_BAD_REQUEST)

@api_view(['GET'])
@permission_classes([IsAuthenticated])
def get_swap_history(request):
    """Get user's swap history"""
    try:
        user_vault = request.user.uservault
        swaps = SwapTransaction.objects.filter(
            user_vault=user_vault
        ).order_by('-created_at')[:50]
        
        serializer = SwapTransactionSerializer(swaps, many=True)
        return Response(serializer.data)
        
    except Exception as e:
        return Response({
            'error': str(e)
        }, status=status.HTTP_400_BAD_REQUEST)

@api_view(['GET'])
@permission_classes([IsAuthenticated])
def get_token_balances(request):
    """Get user's token balances"""
    try:
        user_vault = request.user.uservault
        token_vault = user_vault.token_vault
        
        return Response({
            'sol_balance': str(user_vault.sol_balance),
            'wsol_balance': str(token_vault.wsol_balance),
            'usdc_balance': str(token_vault.usdc_balance),
            'initialized': token_vault.initialized
        })
        
    except Exception as e:
        return Response({
            'error': str(e)
        }, status=status.HTTP_400_BAD_REQUEST)
```

## URL Configuration

```python
# vault/urls.py
from django.urls import path
from . import views

urlpatterns = [
    # Token vault endpoints
    path('token-vault/initialize/', views.initialize_token_vault, name='initialize-token-vault'),
    path('token-vault/balances/', views.get_token_balances, name='token-balances'),
    
    # Swap endpoints
    path('swap/estimate/', views.estimate_swap, name='estimate-swap'),
    path('swap/execute/', views.swap_sol_to_usdc, name='execute-swap'),
    path('swap/history/', views.get_swap_history, name='swap-history'),
    
    # Withdrawal
    path('withdraw/usdc/', views.withdraw_usdc, name='withdraw-usdc'),
]
```

## Celery Tasks

Create async tasks for processing swaps:

```python
# vault/tasks.py
from celery import shared_task
from celery.utils.log import get_task_logger
import asyncio
from decimal import Decimal
from django.utils import timezone
from solders.pubkey import Pubkey
from solders.keypair import Keypair

from .models import SwapTransaction, UserVault, TokenVault
from .services.solana_service import VaultProgramService

logger = get_task_logger(__name__)

@shared_task(bind=True, max_retries=3)
def process_swap_transaction(self, swap_id):
    """Process swap transaction asynchronously"""
    try:
        swap = SwapTransaction.objects.get(id=swap_id)
        
        if swap.status != 'pending':
            logger.info(f"Swap {swap_id} already processed with status: {swap.status}")
            return
        
        swap.status = 'processing'
        swap.save()
        
        user_vault = swap.user_vault
        token_vault = user_vault.token_vault
        
        # Get user keypair (in production, handle this securely)
        user_pubkey = Pubkey.from_string(user_vault.pubkey)
        user_keypair = Keypair()  # Replace with actual user keypair
        
        # Convert amounts to smallest units
        sol_amount_lamports = int(swap.sol_amount * Decimal('1000000000'))
        min_usdc_smallest = int(swap.minimum_usdc_amount * Decimal('1000000'))
        
        # Execute swap
        service = VaultProgramService()
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        
        try:
            # Step 1: Wrap SOL if needed
            if token_vault.wsol_balance < swap.sol_amount:
                wrap_amount = sol_amount_lamports
                tx_hash = loop.run_until_complete(
                    service.wrap_sol(user_pubkey, wrap_amount, user_keypair)
                )
                logger.info(f"Wrapped {swap.sol_amount} SOL, tx: {tx_hash}")
            
            # Step 2: Execute swap
            tx_hash = loop.run_until_complete(
                service.user_swap_sol_to_usdc(
                    user_pubkey,
                    sol_amount_lamports,
                    min_usdc_smallest,
                    user_keypair
                )
            )
            
            # Update swap record
            swap.transaction_hash = tx_hash
            swap.status = 'completed'
            swap.completed_at = timezone.now()
            
            # Calculate actual received amount (would need to read from chain)
            swap.actual_usdc_received = swap.usdc_amount  # Placeholder
            swap.price_per_sol = swap.actual_usdc_received / swap.sol_amount
            
            # Update balances
            user_vault.sol_balance -= swap.sol_amount
            user_vault.save()
            
            token_vault.usdc_balance += swap.actual_usdc_received
            token_vault.save()
            
            swap.save()
            
            logger.info(f"Swap {swap_id} completed successfully, tx: {tx_hash}")
            return tx_hash
            
        finally:
            loop.run_until_complete(service.close())
            loop.close()
            
    except Exception as e:
        logger.error(f"Error processing swap {swap_id}: {str(e)}")
        
        swap = SwapTransaction.objects.get(id=swap_id)
        swap.status = 'failed'
        swap.error_message = str(e)
        swap.save()
        
        # Retry if retries available
        raise self.retry(exc=e, countdown=60)

@shared_task
def update_pool_prices():
    """Periodically update pool prices"""
    from .services.solana_service import OrcaService
    
    service = OrcaService()
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    
    try:
        # Update pool prices in database
        pool_address = os.getenv("ORCA_SOL_USDC_POOL")
        if pool_address:
            price = loop.run_until_complete(
                service.get_pool_price(Pubkey.from_string(pool_address))
            )
            
            # Update OrcaPoolInfo model
            from .models import OrcaPoolInfo
            pool_info, _ = OrcaPoolInfo.objects.get_or_create(
                pool_address=pool_address,
                defaults={
                    'token_a_mint': os.getenv("WSOL_MINT"),
                    'token_b_mint': os.getenv("USDC_MINT_DEVNET"),
                    'tick_spacing': 64,
                }
            )
            pool_info.current_price = price
            pool_info.save()
            
            logger.info(f"Updated pool price: {price}")
            
    finally:
        loop.run_until_complete(service.close())
        loop.close()
```

## Testing

Create tests for the swap functionality:

```python
# vault/tests/test_swap.py
from django.test import TestCase
from django.contrib.auth.models import User
from rest_framework.test import APIClient
from rest_framework import status
from decimal import Decimal
from unittest.mock import patch, MagicMock
from ..models import UserVault, TokenVault, SwapTransaction

class SwapTestCase(TestCase):
    def setUp(self):
        self.client = APIClient()
        self.user = User.objects.create_user(
            username='testuser',
            password='testpass123'
        )
        self.client.force_authenticate(user=self.user)
        
        # Create user vault
        self.user_vault = UserVault.objects.create(
            user=self.user,
            pubkey='7cv2FJpvMmS3ZGysZDB5qVQxXL9nEyVrDaQUXVnzhTpH',
            sol_balance=Decimal('10.0')
        )
        
        # Create token vault
        self.token_vault = TokenVault.objects.create(
            user_vault=self.user_vault,
            initialized=True,
            wsol_balance=Decimal('5.0'),
            usdc_balance=Decimal('100.0')
        )
    
    def test_initialize_token_vault(self):
        """Test token vault initialization"""
        self.token_vault.initialized = False
        self.token_vault.save()
        
        with patch('vault.views.VaultProgramService') as mock_service:
            mock_instance = mock_service.return_value
            mock_instance.initialize_token_vault.return_value = 'mock_tx_hash'
            mock_instance.get_pda_addresses.return_value = {
                'vault_pda': (MagicMock(), 255)
            }
            mock_instance.get_associated_token_address.return_value = MagicMock()
            
            response = self.client.post('/api/vault/token-vault/initialize/')
            
            self.assertEqual(response.status_code, status.HTTP_201_CREATED)
            self.assertIn('transaction', response.data)
    
    def test_estimate_swap(self):
        """Test swap estimation"""
        with patch('vault.views.OrcaService') as mock_service:
            mock_instance = mock_service.return_value
            mock_instance.estimate_swap_output.return_value = {
                'estimated_output': Decimal('405.0'),
                'minimum_output': Decimal('402.975'),
                'price_per_sol': Decimal('40.5'),
                'price_impact': Decimal('0.01')
            }
            
            response = self.client.post('/api/vault/swap/estimate/', {
                'sol_amount': '10.0',
                'slippage_bps': 50
            })
            
            self.assertEqual(response.status_code, status.HTTP_200_OK)
            self.assertEqual(response.data['estimated_usdc'], '405.0')
            self.assertEqual(response.data['minimum_usdc'], '402.975')
    
    def test_execute_swap(self):
        """Test swap execution"""
        with patch('vault.views.OrcaService') as mock_orca:
            mock_orca_instance = mock_orca.return_value
            mock_orca_instance.estimate_swap_output.return_value = {
                'estimated_output': Decimal('405.0'),
                'minimum_output': Decimal('402.975'),
                'price_per_sol': Decimal('40.5'),
                'price_impact': Decimal('0.01')
            }
            
            with patch('vault.views.process_swap_transaction') as mock_task:
                response = self.client.post('/api/vault/swap/execute/', {
                    'sol_amount': '5.0',
                    'slippage_bps': 50
                })
                
                self.assertEqual(response.status_code, status.HTTP_202_ACCEPTED)
                self.assertIn('swap_id', response.data)
                
                # Check swap transaction was created
                swap = SwapTransaction.objects.get(id=response.data['swap_id'])
                self.assertEqual(swap.sol_amount, Decimal('5.0'))
                self.assertEqual(swap.status, 'pending')
                
                # Check task was queued
                mock_task.delay.assert_called_once()
    
    def test_insufficient_balance(self):
        """Test swap with insufficient balance"""
        response = self.client.post('/api/vault/swap/execute/', {
            'sol_amount': '100.0',  # More than available
            'slippage_bps': 50
        })
        
        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertIn('Insufficient SOL balance', response.data['error'])
    
    def test_withdraw_usdc(self):
        """Test USDC withdrawal"""
        with patch('vault.views.VaultProgramService') as mock_service:
            mock_instance = mock_service.return_value
            mock_instance.withdraw_usdc.return_value = 'mock_tx_hash'
            
            response = self.client.post('/api/vault/withdraw/usdc/', {
                'amount': '50.0'
            })
            
            self.assertEqual(response.status_code, status.HTTP_200_OK)
            self.assertIn('transaction', response.data)
            
            # Check balance was updated
            self.token_vault.refresh_from_db()
            self.assertEqual(self.token_vault.usdc_balance, Decimal('50.0'))
```

## Deployment Checklist

### Environment Setup
- [ ] Configure all environment variables
- [ ] Set up Solana RPC endpoint (use dedicated RPC for production)
- [ ] Configure Celery with Redis/RabbitMQ
- [ ] Set up monitoring (Sentry, DataDog, etc.)

### Security
- [ ] Implement proper keypair management (use HSM or KMS)
- [ ] Add rate limiting to swap endpoints
- [ ] Implement transaction signing on separate service
- [ ] Add audit logging for all transactions
- [ ] Set up alerts for failed swaps

### Database
- [ ] Run migrations: `python manage.py migrate`
- [ ] Create indexes on frequently queried fields
- [ ] Set up database backups

### Testing
- [ ] Run unit tests: `python manage.py test vault.tests`
- [ ] Test on DevNet with real transactions
- [ ] Load testing for API endpoints
- [ ] Test error handling and retry logic

### Monitoring
- [ ] Set up transaction monitoring dashboard
- [ ] Configure alerts for:
  - Failed swaps
  - Low balance warnings
  - Price deviations
  - High slippage events

## Common Issues and Solutions

### Issue: "Token vault not initialized"
**Solution**: Ensure `initialize_token_vault` is called for each user before swapping.

### Issue: "Insufficient SOL for fees"
**Solution**: Maintain minimum 0.01 SOL in vault for transaction fees.

### Issue: "Slippage tolerance exceeded"
**Solution**: Increase slippage_bps or wait for better market conditions.

### Issue: "Transaction simulation failed"
**Solution**: Check account balances and ensure all PDAs are properly initialized.

## API Reference

### Initialize Token Vault
```
POST /api/vault/token-vault/initialize/
Authorization: Bearer <token>
```

### Estimate Swap
```
POST /api/vault/swap/estimate/
Authorization: Bearer <token>
Body: {
    "sol_amount": "10.0",
    "slippage_bps": 50
}
```

### Execute Swap
```
POST /api/vault/swap/execute/
Authorization: Bearer <token>
Body: {
    "sol_amount": "10.0",
    "minimum_usdc_amount": "400.0",  // optional
    "slippage_bps": 50
}
```

### Withdraw USDC
```
POST /api/vault/withdraw/usdc/
Authorization: Bearer <token>
Body: {
    "amount": "100.0",
    "recipient_address": "..."  // optional
}
```

### Get Token Balances
```
GET /api/vault/token-vault/balances/
Authorization: Bearer <token>
```

### Get Swap History
```
GET /api/vault/swap/history/
Authorization: Bearer <token>
```

## Next Steps

1. **Production Deployment**:
   - Use mainnet-beta endpoints
   - Implement proper keypair management
   - Add comprehensive monitoring

2. **Advanced Features**:
   - Support multiple token pairs
   - Implement limit orders
   - Add DCA (Dollar Cost Averaging) swaps
   - Integrate multiple DEXs for best price

3. **Optimization**:
   - Implement transaction batching
   - Add caching for pool prices
   - Optimize RPC calls with WebSockets

4. **User Experience**:
   - Add WebSocket support for real-time updates
   - Implement swap notifications
   - Add transaction history export

This implementation guide provides a complete foundation for integrating the SOL to USDC swap functionality into your Django backend. Adjust the code according to your specific requirements and security standards.