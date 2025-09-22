# IDL Updates Summary for vault_app

## Overview
The `vault_app.json` IDL file has been updated with new instructions and account types to support SOL to USDC swap functionality via Orca Whirlpool integration.

## New Instructions Added

### Token Vault Management
1. **initialize_token_vault** - Initializes token vault with wSOL and USDC ATAs
2. **wrap_sol** - Converts native SOL to wrapped SOL (wSOL)
3. **unwrap_sol** - Converts wSOL back to native SOL

### Swap Operations
4. **swap_sol_to_usdc** - Direct swap using Orca Whirlpool CPI
5. **user_swap_sol_to_usdc** - User-initiated swap with balance tracking
6. **withdraw_usdc** - Withdraw USDC from vault to user wallet

## New Account Types

### TokenVault
```typescript
{
  authority: PublicKey;      // Vault authority
  vaultWsolAta: PublicKey;   // Vault's wSOL Associated Token Account
  vaultUsdcAta: PublicKey;   // Vault's USDC Associated Token Account
  totalWsol: BN;             // Total wSOL in vault
  totalUsdc: BN;             // Total USDC in vault
  bump: number;              // PDA bump
}
```

### UserTokenAccount
```typescript
{
  owner: PublicKey;           // User pubkey
  wsolBalance: BN;           // User's wSOL balance
  usdcBalance: BN;           // User's USDC balance
  lastSwapTimestamp: BN;     // Last swap timestamp
  totalSwapped: BN;          // Total amount swapped
  bump: number;              // PDA bump
}
```

### SwapState
```typescript
{
  totalSolSwapped: BN;       // Total SOL swapped
  totalUsdcReceived: BN;     // Total USDC received
  lastSwapPrice: BN;         // Last swap price
  swapCount: BN;             // Number of swaps
}
```

## Updated Error Codes
- InsufficientSOLBalance
- InsufficientWSOLBalance
- SlippageExceeded
- InvalidPoolConfiguration
- SwapFailed
- WrapFailed
- UnwrapFailed
- InvalidTokenMint
- TokenAccountNotInitialized

## Integration Requirements for Django Backend

### 1. Update Anchor Client
The Django backend needs to parse the new IDL to interact with the updated program:

```python
# Load updated IDL
import json
with open('vault_app.json', 'r') as f:
    idl = json.load(f)

# Access new instructions
swap_instruction = next(i for i in idl['instructions'] if i['name'] == 'userSwapSolToUsdc')
```

### 2. New Instruction Parameters

**wrap_sol**
- amount: u64 (lamports to wrap)

**user_swap_sol_to_usdc**
- sol_amount: u64 (lamports)
- min_usdc_out: u64 (minimum USDC in smallest unit)

**swap_sol_to_usdc** (for direct Orca integration)
- amount_in: u64
- minimum_amount_out: u64
- sqrt_price_limit: u128
- amount_specified_is_input: bool
- a_to_b: bool (true for SOL→USDC)

**withdraw_usdc**
- amount: u64 (USDC in smallest unit, 6 decimals)

### 3. Account Requirements

Each new instruction requires specific accounts in order:

**initialize_token_vault accounts:**
1. token_vault (init, writable)
2. vault_pda
3. vault_wsol_ata (init, writable)
4. vault_usdc_ata (init, writable)
5. wsol_mint
6. usdc_mint
7. payer (signer, writable)
8. authority (signer)
9. token_program
10. associated_token_program
11. system_program

**user_swap_sol_to_usdc accounts:**
1. user_vault (writable)
2. user_token_account (init if needed, writable)
3. token_vault (writable)
4. vault_wsol_ata (writable)
5. vault_usdc_ata (writable)
6. user (signer, writable)
7. owner (signer)
8. system_program

### 4. PDA Seeds
New PDAs use these seeds:
- Token Vault: `[b"token_vault"]`
- User Token Account: `[b"user_token_account", user_pubkey]`
- Swap State: `[b"swap_state"]`

## Frontend Integration

The frontend TypeScript/JavaScript SDK should:

1. **Update IDL import:**
```typescript
import idl from './vault_app.json';
```

2. **Add new methods to SDK:**
```typescript
async initializeTokenVault(): Promise<string>
async wrapSol(amount: BN): Promise<string>
async swapSolToUsdc(amount: BN, minUsdc: BN): Promise<string>
async withdrawUsdc(amount: BN): Promise<string>
async getTokenBalances(): Promise<TokenBalances>
```

3. **Handle new account types:**
```typescript
interface TokenVault {
  authority: PublicKey;
  vaultWsolAta: PublicKey;
  vaultUsdcAta: PublicKey;
  totalWsol: BN;
  totalUsdc: BN;
}
```

## Testing the Updated Program

1. **Deploy to DevNet:**
```bash
anchor deploy --provider.cluster devnet
```

2. **Test initialization:**
```bash
# Initialize token vault
anchor test --skip-local-validator
```

3. **Test swap flow:**
- Initialize token vault
- Deposit SOL
- Wrap SOL to wSOL
- Execute swap
- Check USDC balance
- Withdraw USDC

## Migration Notes

### For Existing Users
1. Existing vault functionality remains unchanged
2. Users need to call `initialize_token_vault` once before using swap features
3. SOL deposits continue to work as before

### Breaking Changes
None - all existing instructions maintain backward compatibility

## File Location
The updated IDL file is located at: `/home/jhonydev/mytestproject/vault_app.json`

This IDL should be:
1. Committed to version control
2. Shared with frontend developers
3. Used to update Django backend service layer
4. Deployed alongside the program to DevNet/Mainnet