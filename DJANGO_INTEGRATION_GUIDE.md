# Django Backend Integration Guide for Vault Program with Swap

## Overview
This guide provides instructions for integrating the deployed Solana vault program (with SOL to USDC swap functionality) into the Django backend.

## Deployed Program Information

### DevNet Details
```python
PROGRAM_ID = "5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL"
RPC_URL = "https://api.devnet.solana.com"
WSOL_MINT = "So11111111111111111111111111111111111112"
USDC_MINT_DEVNET = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
```

### Explorer Link
[View Program on Solana Explorer](https://explorer.solana.com/address/5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL?cluster=devnet)

## Required Files

### 1. IDL File
- **Location**: `vault_app.json`
- **Purpose**: Interface Definition Language file that describes all program instructions and accounts
- **Usage**: Parse this file to build transactions in Django

### 2. Implementation Guide
- **Location**: `DJANGO_BACKEND_IMPLEMENTATION_GUIDE.md`
- **Purpose**: Complete implementation examples including models, services, and API endpoints

## New Instructions Available

### Token Operations
1. **initialize_token_vault** - One-time setup for token accounts
2. **wrap_sol** - Convert SOL to wSOL
3. **unwrap_sol** - Convert wSOL back to SOL

### Swap Operations
4. **user_swap_sol_to_usdc** - Execute SOL to USDC swap
5. **withdraw_usdc** - Withdraw USDC to user wallet

### Existing Operations (unchanged)
- initialize_vault
- initialize_user_vault
- deposit
- withdraw
- get_user_balance
- get_vault_stats

## Quick Start Integration

### 1. Install Required Packages
```bash
pip install solana==0.30.2 anchorpy==0.18.0 solders==0.18.1
```

### 2. Update Environment Variables
```env
# .env
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
WSOL_MINT=So11111111111111111111111111111111111112
USDC_MINT_DEVNET=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU
CLUSTER=devnet
```

### 3. Load IDL in Django
```python
import json
from pathlib import Path

# Load IDL
with open('vault_app.json', 'r') as f:
    IDL = json.load(f)

# Access instruction schemas
swap_instruction = next(
    i for i in IDL['instructions'] 
    if i['name'] == 'userSwapSolToUsdc'
)
```

### 4. PDA Derivation Seeds
```python
# PDA seeds for account derivation
SEEDS = {
    'vault': [b"vault"],
    'vault_pda': [b"vault_pda"],
    'user_vault': [b"user_vault", user_pubkey_bytes],
    'token_vault': [b"token_vault"],
    'user_token_account': [b"user_token_account", user_pubkey_bytes],
    'swap_state': [b"swap_state"]
}
```

## New API Endpoints to Implement

### 1. Initialize Token Vault
```
POST /api/vault/token-vault/initialize/
```
First-time setup for each user to enable swap functionality.

### 2. Estimate Swap
```
POST /api/vault/swap/estimate/
Body: {
    "sol_amount": "1.5",
    "slippage_bps": 50
}
```
Returns estimated USDC output.

### 3. Execute Swap
```
POST /api/vault/swap/execute/
Body: {
    "sol_amount": "1.5",
    "minimum_usdc_amount": "60.0",
    "slippage_bps": 50
}
```
Executes the swap transaction.

### 4. Withdraw USDC
```
POST /api/vault/withdraw/usdc/
Body: {
    "amount": "100.0"
}
```
Withdraws USDC to user's wallet.

### 5. Get Token Balances
```
GET /api/vault/token-vault/balances/
```
Returns SOL, wSOL, and USDC balances.

## Transaction Building Example

### Initialize Token Vault
```python
from solders.pubkey import Pubkey
from solders.instruction import Instruction
from solana.transaction import Transaction

async def initialize_token_vault(user_pubkey: Pubkey):
    # Derive PDAs
    token_vault_pda = Pubkey.find_program_address(
        [b"token_vault"],
        Pubkey.from_string(PROGRAM_ID)
    )[0]
    
    vault_pda = Pubkey.find_program_address(
        [b"vault_pda"],
        Pubkey.from_string(PROGRAM_ID)
    )[0]
    
    # Get ATAs
    vault_wsol_ata = get_associated_token_address(
        WSOL_MINT, vault_pda
    )
    vault_usdc_ata = get_associated_token_address(
        USDC_MINT_DEVNET, vault_pda
    )
    
    # Build instruction
    accounts = [
        # token_vault (writable)
        # vault_pda
        # vault_wsol_ata (writable)
        # vault_usdc_ata (writable)
        # wsol_mint
        # usdc_mint
        # payer (signer, writable)
        # authority (signer)
        # token_program
        # associated_token_program
        # system_program
    ]
    
    # Get discriminator from IDL
    discriminator = get_instruction_discriminator("initializeTokenVault")
    
    ix = Instruction(
        program_id=PROGRAM_ID,
        accounts=accounts,
        data=discriminator
    )
    
    return Transaction().add(ix)
```

### Execute Swap
```python
async def user_swap_sol_to_usdc(
    user_pubkey: Pubkey,
    sol_amount: float,
    min_usdc_out: float
):
    # Convert to lamports and smallest USDC unit
    sol_lamports = int(sol_amount * 10**9)
    min_usdc = int(min_usdc_out * 10**6)
    
    # Derive all required PDAs
    pdas = derive_all_pdas(user_pubkey)
    
    # Build instruction data
    import struct
    discriminator = get_instruction_discriminator("userSwapSolToUsdc")
    data = discriminator + struct.pack("<QQ", sol_lamports, min_usdc)
    
    # Create and send transaction
    # ...
```

## Important Considerations

### 1. Account Initialization Order
1. First: `initialize_vault` (if not done)
2. Second: `initialize_user_vault` (if not done)
3. Third: `initialize_token_vault` (once per user)
4. Then: User can swap

### 2. Balance Conversions
- SOL: 1 SOL = 10^9 lamports
- USDC: 1 USDC = 10^6 smallest units
- Always convert user input to smallest units

### 3. Error Handling
```python
# New error codes to handle
ERROR_CODES = {
    "InsufficientSOLBalance": "User doesn't have enough SOL",
    "InsufficientWSOLBalance": "Not enough wrapped SOL",
    "SlippageExceeded": "Price moved beyond slippage tolerance",
    "WrapFailed": "Failed to wrap SOL",
    "TokenAccountNotInitialized": "Token vault not initialized"
}
```

### 4. Testing on DevNet
```python
# Test configuration
TEST_CONFIG = {
    "test_user_keypair": "path/to/test-keypair.json",
    "test_sol_amount": 0.1,  # Start with small amounts
    "test_slippage_bps": 100,  # 1% slippage for testing
}

# DevNet faucet for testing
DEVNET_FAUCET = "https://api.devnet.solana.com/faucet"
```

## Database Schema Updates

### New Models Required
```python
class TokenVault(models.Model):
    user_vault = models.OneToOneField(UserVault, on_delete=models.CASCADE)
    wsol_ata = models.CharField(max_length=44)
    usdc_ata = models.CharField(max_length=44)
    wsol_balance = models.DecimalField(max_digits=20, decimal_places=9)
    usdc_balance = models.DecimalField(max_digits=20, decimal_places=6)
    initialized = models.BooleanField(default=False)

class SwapTransaction(models.Model):
    user_vault = models.ForeignKey(UserVault, on_delete=models.CASCADE)
    transaction_hash = models.CharField(max_length=128)
    sol_amount = models.DecimalField(max_digits=20, decimal_places=9)
    usdc_amount = models.DecimalField(max_digits=20, decimal_places=6)
    status = models.CharField(max_length=20)
    created_at = models.DateTimeField(auto_now_add=True)
```

## Monitoring & Debugging

### View Program Logs
```bash
solana logs 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL --url devnet
```

### Check Transaction Status
```python
async def check_transaction(signature: str):
    client = AsyncClient(RPC_URL)
    result = await client.get_transaction(signature)
    return result
```

## Common Issues & Solutions

### Issue: "Token vault not initialized"
**Solution**: Call `initialize_token_vault` for the user first

### Issue: "Insufficient SOL for fees"
**Solution**: Ensure user has at least 0.01 SOL for transaction fees

### Issue: "Custom program error: 0x1"
**Solution**: Check that all accounts are passed in correct order per IDL

### Issue: "Account does not exist"
**Solution**: Ensure PDAs are derived with correct seeds

## Support Resources

1. **Full Implementation Guide**: See `DJANGO_BACKEND_IMPLEMENTATION_GUIDE.md`
2. **IDL Reference**: Check `vault_app.json` for exact account orders
3. **Program Explorer**: https://explorer.solana.com/address/5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL?cluster=devnet
4. **DevNet Faucet**: https://faucet.solana.com (for test SOL)

## Next Steps

1. ✅ Load and parse the IDL file
2. ✅ Update environment variables
3. ✅ Create new database models
4. ✅ Implement token vault initialization endpoint
5. ✅ Implement swap estimation endpoint
6. ✅ Implement swap execution with Celery
7. ✅ Test with small amounts on DevNet
8. ✅ Add monitoring and error handling

## Contact for Issues
If you encounter issues with the program itself, check the program logs or contact the Solana program developer with:
- Program ID: `5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL`
- Error messages from logs
- Transaction signatures that failed