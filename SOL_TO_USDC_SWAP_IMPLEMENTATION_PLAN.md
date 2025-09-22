# SOL to USDC Swap Implementation Plan

## Overview
This document outlines the implementation plan for adding SOL to USDC swap functionality to the existing vault program using Orca's Whirlpool on Solana DevNet.

## Current System Analysis

### Existing Functionality
- **Vault Management**: Initialize vault with authority
- **User Vault Accounts**: Per-user vault tracking (deposits, withdrawals, balances)
- **Native SOL Operations**: Deposit and withdraw SOL (lamports)
- **PDA Structure**: 
  - `vault`: Main vault account (stores metadata)
  - `vault_pda`: SOL holding account
  - `user_vault`: Per-user tracking account

### Current Limitations
- Only handles native SOL (lamports)
- No SPL token support
- No DEX integration
- No wrapped SOL (wSOL) functionality

## Implementation Phases

### Phase 1: SPL Token Foundation
**Goal**: Add SPL token support to the vault program

#### 1.1 Dependencies
```toml
# Add to Cargo.toml
anchor-spl = "0.31.1"
spl-token = "4.0.0"
spl-associated-token-account = "3.0.2"
```

#### 1.2 New Account Structures
```rust
// Token vault configuration
pub struct TokenVault {
    pub authority: Pubkey,
    pub vault_wsol_ata: Pubkey,  // Vault's wSOL ATA
    pub vault_usdc_ata: Pubkey,  // Vault's USDC ATA
    pub total_wsol: u64,
    pub total_usdc: u64,
    pub bump: u8,
}

// User token balances
pub struct UserTokenAccount {
    pub owner: Pubkey,
    pub wsol_balance: u64,
    pub usdc_balance: u64,
    pub bump: u8,
}
```

### Phase 2: Wrapped SOL (wSOL) Operations
**Goal**: Enable SOL ↔ wSOL conversions within the vault

#### 2.1 New Instructions

##### `wrap_sol`
**Purpose**: Convert native SOL in vault to wSOL
```rust
pub fn wrap_sol(ctx: Context<WrapSol>, amount: u64) -> Result<()>
```

**Accounts Required**:
- `vault_pda`: Source of SOL
- `wsol_mint`: Native mint (So11111111111111111111111111111111111112)
- `vault_wsol_ata`: Vault's wSOL Associated Token Account
- `token_program`: SPL Token Program
- `associated_token_program`: ATA Program
- `system_program`: System Program

**Process**:
1. Create vault's wSOL ATA if not exists
2. Transfer SOL from vault_pda to wSOL ATA
3. Sync native account to update wSOL balance
4. Update vault tracking

##### `unwrap_sol`
**Purpose**: Convert wSOL back to native SOL
```rust
pub fn unwrap_sol(ctx: Context<UnwrapSol>, amount: u64) -> Result<()>
```

**Accounts Required**:
- `vault_wsol_ata`: Source of wSOL
- `vault_pda`: Destination for SOL
- `token_program`: SPL Token Program

**Process**:
1. Close wSOL account partially
2. Transfer SOL back to vault_pda
3. Update vault tracking

### Phase 3: Orca Whirlpool Integration
**Goal**: Enable SOL → USDC swaps via Orca

#### 3.1 Dependencies
```toml
# Add Orca/Whirlpool types
whirlpool = { version = "0.1.0", features = ["cpi"] }
```

#### 3.2 Pool Configuration
```rust
pub struct OrcaPoolConfig {
    pub pool_address: Pubkey,        // Orca SOL/USDC pool on DevNet
    pub token_vault_a: Pubkey,       // Pool's wSOL vault
    pub token_vault_b: Pubkey,       // Pool's USDC vault
    pub tick_array_0: Pubkey,        // Tick arrays for price
    pub tick_array_1: Pubkey,
    pub tick_array_2: Pubkey,
    pub oracle: Pubkey,              // Price oracle
}
```

#### 3.3 Swap Instruction

##### `swap_sol_to_usdc`
**Purpose**: Swap wSOL for USDC using Orca Whirlpool
```rust
pub fn swap_sol_to_usdc(
    ctx: Context<SwapSolToUsdc>,
    amount_in: u64,
    minimum_amount_out: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool  // true for SOL→USDC
) -> Result<()>
```

**Accounts Required**:
- **Vault Accounts**:
  - `vault`: Main vault account
  - `vault_pda`: Vault PDA (signer)
  - `vault_wsol_ata`: Source wSOL
  - `vault_usdc_ata`: Destination USDC
  
- **Orca Whirlpool Accounts**:
  - `whirlpool_program`: Orca program ID
  - `whirlpool`: Pool state account
  - `token_vault_a`: Pool's wSOL vault
  - `token_vault_b`: Pool's USDC vault
  - `tick_array_0/1/2`: Price tick arrays
  - `oracle`: Price oracle account

- **Token Programs**:
  - `token_program`: SPL Token Program
  - `associated_token_program`: ATA Program

**Process**:
1. Validate swap parameters
2. Prepare CPI context with vault_pda as signer
3. Execute Orca swap CPI
4. Update vault token balances
5. Emit swap event with details

### Phase 4: User-Facing Operations
**Goal**: Allow users to swap their deposited SOL to USDC

#### 4.1 High-Level User Flow

##### `user_swap_sol_to_usdc`
**Purpose**: User initiates swap of their SOL balance to USDC
```rust
pub fn user_swap_sol_to_usdc(
    ctx: Context<UserSwapSolToUsdc>,
    amount: u64,
    min_usdc_out: u64
) -> Result<()>
```

**Process**:
1. Verify user has sufficient SOL balance
2. Wrap user's SOL allocation to wSOL
3. Execute swap via Orca
4. Update user's balances (decrease SOL, increase USDC)
5. Return swap details

##### `withdraw_usdc`
**Purpose**: Withdraw USDC from vault to user's wallet
```rust
pub fn withdraw_usdc(
    ctx: Context<WithdrawUsdc>,
    amount: u64
) -> Result<()>
```

### Phase 5: Safety & Error Handling

#### 5.1 Error Codes
```rust
pub enum SwapErrorCode {
    InsufficientSOLBalance,
    InsufficientWSOLBalance,
    SlippageExceeded,
    InvalidPoolConfiguration,
    SwapFailed,
    WrapFailed,
    UnwrapFailed,
    MathOverflow,
}
```

#### 5.2 Safety Features
- Slippage protection (minimum output amounts)
- Balance validations before operations
- Atomic transactions (all or nothing)
- Detailed error messages and events

## DevNet Configuration

### Required DevNet Addresses
```rust
// Token Mints (DevNet)
const WSOL_MINT: &str = "So11111111111111111111111111111111111112";
const USDC_MINT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";  // DevNet USDC

// Orca Whirlpool Program (DevNet)
const ORCA_WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";

// SOL/USDC Pool (DevNet) - To be configured
const SOL_USDC_POOL: &str = "TBD - Need to find or create DevNet pool";
```

## Testing Strategy

### Unit Tests
1. Test wrap/unwrap SOL operations
2. Test balance updates
3. Test error conditions

### Integration Tests
1. Full flow: Deposit SOL → Wrap → Swap → Withdraw USDC
2. Slippage scenarios
3. Insufficient balance scenarios
4. Multiple user scenarios

### DevNet Testing
1. Deploy to DevNet
2. Test with real DevNet tokens
3. Monitor gas costs
4. Validate against Orca pools

## Frontend Integration Points

### New SDK Methods
```typescript
// Swap operations
async swapSolToUsdc(amount: number, minUsdc: number): Promise<TransactionSignature>
async getUserUsdcBalance(): Promise<number>
async withdrawUsdc(amount: number): Promise<TransactionSignature>

// Pool information
async getPoolPrice(): Promise<number>
async estimateSwapOutput(amountIn: number): Promise<number>
```

### UI Components Needed
1. Swap interface (amount input, slippage settings)
2. Balance display (SOL and USDC)
3. Price display and impact
4. Transaction history with swap details

## Implementation Timeline

### Week 1: Foundation
- [ ] Add SPL token dependencies
- [ ] Implement token vault structures
- [ ] Create wrap/unwrap SOL instructions
- [ ] Write unit tests for SOL operations

### Week 2: Orca Integration
- [ ] Add Orca dependencies
- [ ] Find/configure DevNet pool
- [ ] Implement swap instruction
- [ ] Handle CPI calls to Orca

### Week 3: User Operations & Testing
- [ ] Implement user swap functions
- [ ] Add USDC withdrawal
- [ ] Complete integration tests
- [ ] Deploy to DevNet

### Week 4: Frontend & Polish
- [ ] Update SDK/IDL
- [ ] Implement frontend components
- [ ] End-to-end testing
- [ ] Documentation

## Risks & Mitigations

### Technical Risks
1. **Orca Pool Availability on DevNet**
   - Mitigation: May need to create test pool or use alternative DEX

2. **CPI Complexity**
   - Mitigation: Start with simple swaps, add features incrementally

3. **Gas Costs**
   - Mitigation: Optimize account usage, batch operations where possible

### Security Considerations
1. **Slippage Attacks**
   - Always enforce minimum output amounts
   - Add max slippage parameters

2. **Authority Management**
   - Vault PDA must be only signer for swaps
   - User permissions strictly enforced

3. **Balance Tracking**
   - Atomic updates to prevent inconsistencies
   - Regular reconciliation checks

## Next Steps
1. Review and approve implementation plan
2. Set up development environment with Orca SDK
3. Begin Phase 1 implementation
4. Create detailed test scenarios