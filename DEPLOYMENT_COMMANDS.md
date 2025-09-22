# DevNet Deployment Commands

## Prerequisites Check

```bash
# 1. Check Solana CLI version
solana --version

# 2. Check Anchor version
anchor --version

# 3. Check current network configuration
solana config get

# 4. Check wallet balance
solana balance
```

## Step 1: Configure Solana CLI for DevNet

```bash
# Set cluster to DevNet
solana config set --url https://api.devnet.solana.com

# Verify configuration
solana config get

# Generate a new keypair (if needed)
# WARNING: Save the recovery phrase!
solana-keygen new --outfile ~/.config/solana/devnet-wallet.json

# Or use existing keypair
solana config set --keypair ~/.config/solana/id.json

# Check your wallet address
solana address

# Get DevNet SOL from faucet (2 SOL at a time)
solana airdrop 2

# Check balance
solana balance
```

## Step 2: Update Anchor.toml Configuration

Make sure your `Anchor.toml` file is configured for DevNet:

```toml
[features]
seeds = false
skip-lint = false

[programs.devnet]
vault_app = "5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

## Step 3: Build the Program

```bash
# Clean previous builds
rm -rf target/

# Build the program
anchor build

# This will:
# - Compile the Rust program
# - Generate the IDL file
# - Create deployable .so file in target/deploy/
```

## Step 4: Deploy to DevNet

```bash
# Deploy the program
anchor deploy --provider.cluster devnet

# Or explicitly specify the program
anchor deploy --program-name vault_app --provider.cluster devnet

# Alternative: Deploy with Solana CLI directly
solana program deploy target/deploy/vault_app.so --program-id 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
```

## Step 5: Verify Deployment

```bash
# Check program is deployed
solana program show 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL

# View program logs (useful for debugging)
solana logs 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL

# Check your program's account
solana account 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
```

## Step 6: Initialize On-Chain Accounts

After deployment, you need to initialize the vault accounts:

```bash
# Run initialization script (if you have one)
anchor run initialize

# Or use Anchor test to initialize
anchor test --skip-local-validator --provider.cluster devnet
```

## Complete Deployment Script

Create a `deploy.sh` script:

```bash
#!/bin/bash

echo "🚀 Starting DevNet Deployment..."

# Configuration
echo "📝 Configuring Solana CLI for DevNet..."
solana config set --url https://api.devnet.solana.com

# Check balance
BALANCE=$(solana balance | cut -d' ' -f1)
echo "💰 Current balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 0.5" | bc -l) )); then
    echo "⚠️  Low balance! Requesting airdrop..."
    solana airdrop 2
    sleep 5
    echo "✅ New balance: $(solana balance)"
fi

# Build
echo "🔨 Building program..."
anchor build

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"

# Deploy
echo "📤 Deploying to DevNet..."
anchor deploy --provider.cluster devnet

if [ $? -ne 0 ]; then
    echo "❌ Deployment failed!"
    exit 1
fi

echo "✅ Deployment successful!"

# Verify
echo "🔍 Verifying deployment..."
PROGRAM_ID="5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL"
solana program show $PROGRAM_ID

echo "🎉 Deployment complete!"
echo "📋 Program ID: $PROGRAM_ID"
echo "🌐 View on Explorer: https://explorer.solana.com/address/$PROGRAM_ID?cluster=devnet"
```

Make it executable:
```bash
chmod +x deploy.sh
./deploy.sh
```

## Troubleshooting Common Issues

### Issue: "Insufficient funds"
```bash
# Request multiple airdrops (max 2 SOL each)
solana airdrop 2
sleep 10
solana airdrop 2
```

### Issue: "Program already deployed"
```bash
# If you need to upgrade the program
solana program deploy target/deploy/vault_app.so --program-id 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL --upgrade-authority ~/.config/solana/id.json
```

### Issue: "Transaction too large"
```bash
# Increase compute budget
solana program deploy target/deploy/vault_app.so --with-compute-unit-price 1 --max-sign-attempts 50
```

### Issue: Build fails with dependency errors
```bash
# Clear Cargo cache and rebuild
cargo clean
rm Cargo.lock
anchor build
```

## Post-Deployment Steps

### 1. Save Deployment Information
```bash
# Create deployment record
cat > deployment-devnet.json << EOF
{
  "cluster": "devnet",
  "programId": "5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL",
  "deployedAt": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "deployer": "$(solana address)",
  "idl": "vault_app.json"
}
EOF
```

### 2. Update Frontend Configuration
```javascript
// Update your frontend config
const PROGRAM_ID = new PublicKey("5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL");
const CLUSTER = "devnet";
const RPC_URL = "https://api.devnet.solana.com";
```

### 3. Update Django Backend
```python
# Update .env
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
CLUSTER=devnet
```

### 4. Initialize Program Accounts
```typescript
// Initialize vault and token vault
import { Program, AnchorProvider, web3 } from "@coral-xyz/anchor";

const provider = AnchorProvider.env();
const program = new Program(idl, programId, provider);

// Initialize main vault
await program.methods
  .initializeVault(authority)
  .accounts({...})
  .rpc();

// Initialize token vault
await program.methods
  .initializeTokenVault()
  .accounts({...})
  .rpc();
```

## Monitoring and Logs

### Watch Real-time Logs
```bash
# Stream program logs
solana logs 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL --url devnet

# In another terminal, run your transactions
```

### View on Solana Explorer
Open in browser:
```
https://explorer.solana.com/address/5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL?cluster=devnet
```

## Quick Command Summary

```bash
# Complete deployment in order:
solana config set --url https://api.devnet.solana.com
solana airdrop 2
anchor build
anchor deploy --provider.cluster devnet
solana program show 5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL
```

## Important DevNet Addresses

- **Your Program**: `5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL`
- **wSOL Mint**: `So11111111111111111111111111111111111112`
- **DevNet USDC**: `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
- **Orca Whirlpool**: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`

## Next Steps After Deployment

1. ✅ Test all instructions using Anchor tests
2. ✅ Initialize token vault for test users
3. ✅ Test swap functionality with small amounts
4. ✅ Monitor program logs for errors
5. ✅ Update frontend to point to DevNet
6. ✅ Document the deployment for team

## Costs

- Initial deployment: ~2-5 SOL (recoverable when program is closed)
- Each transaction: ~0.00025 SOL
- Account rent: ~0.002 SOL per account