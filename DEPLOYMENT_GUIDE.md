# Vault App - Deployment Guide

## Overview
This guide walks you through deploying and testing the Vault program on Solana, starting with devnet testing before mainnet deployment.

## Prerequisites
- Solana CLI installed (`solana --version`)
- Anchor framework installed (`anchor --version`)
- Rust installed (`rustc --version`)
- Python 3.8+ with pip (for client testing)
- At least 2 SOL in your wallet for deployment

## 📝 Project Structure
```
mytestproject/
├── programs/mytestproject/src/lib.rs  # Vault program code
├── client_devnet.py                   # Devnet test client
├── client_example.py                  # Mainnet production client
├── deploy_devnet.sh                   # Automated devnet deployment
├── Anchor.toml                        # Anchor configuration
└── target/idl/vault_app.json         # Program IDL (after build)
```

## 🧪 Phase 1: Devnet Testing

### Step 1: Deploy to Devnet

**Option A: Automated Deployment (Recommended)**
```bash
# Run the automated deployment script
./deploy_devnet.sh
```

This script will:
- Switch to devnet
- Request airdrop if needed
- Build the program
- Deploy to devnet
- Update all configuration files

**Option B: Manual Deployment**
```bash
# 1. Switch to devnet
solana config set --url https://api.devnet.solana.com

# 2. Check balance and request airdrop if needed
solana balance
solana airdrop 2

# 3. Build the program
anchor build

# 4. Get the program ID
solana address -k target/deploy/vault_app-keypair.json

# 5. Update the program ID in lib.rs (replace YOUR_PROGRAM_ID)
# Edit programs/mytestproject/src/lib.rs
# Update: declare_id!("YOUR_PROGRAM_ID");

# 6. Rebuild with correct ID
anchor build

# 7. Deploy to devnet
anchor deploy --provider.cluster devnet
```

### Step 2: Test on Devnet

**Python Client Testing:**
```bash
# Install dependencies
pip install -r requirements.txt

# Run the comprehensive test suite
python client_devnet.py
```

The test suite will:
1. Request devnet SOL airdrop
2. Initialize the vault
3. Deposit 0.1 SOL
4. Check vault balance
5. Withdraw 0.05 SOL
6. Verify all operations

**TypeScript Testing (Optional):**
```bash
# Install dependencies
npm install

# Run Anchor tests
anchor test --provider.cluster devnet
```

### Step 3: Verify on Explorer
After deployment, verify your program on Solana Explorer:
```
https://explorer.solana.com/address/YOUR_PROGRAM_ID?cluster=devnet
```

## 🚀 Phase 2: Mainnet Deployment

### Step 1: Prepare for Mainnet

1. **Update Anchor.toml for mainnet:**
```toml
[provider]
cluster = "mainnet-beta"

[programs.mainnet]
vault_app = "YOUR_PROGRAM_ID"
```

2. **Ensure sufficient SOL balance:**
```bash
# Switch to mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Check balance (need ~2-3 SOL for deployment)
solana balance
```

### Step 2: Deploy to Mainnet

```bash
# Build the program
anchor build

# Deploy to mainnet
anchor deploy --provider.cluster mainnet-beta
```

### Step 3: Production Client Usage

**Python/Django Integration:**
```python
# Use client_example.py for mainnet
from client_example import VaultClient

client = VaultClient()
await client.connect()
await client.deposit(0.1)  # Deposit 0.1 SOL
balance = await client.get_vault_balance()
```

## 📊 Monitoring & Management

### View Program Logs
```bash
# Devnet
solana logs YOUR_PROGRAM_ID --url devnet

# Mainnet
solana logs YOUR_PROGRAM_ID --url mainnet-beta
```

### Check Program Status
```bash
# Get program info
solana program show YOUR_PROGRAM_ID --url devnet
```

### Upgrade Program (if needed)
```bash
# Deploy upgrade
anchor upgrade target/deploy/vault_app.so --program-id YOUR_PROGRAM_ID
```

## 🔑 Security Considerations

1. **Keypair Management:**
   - Never commit keypairs to git
   - Use environment variables for production
   - Store keypairs securely

2. **Authority Control:**
   - Only the initialized authority can withdraw
   - Consider using multisig for production

3. **Audit Before Mainnet:**
   - Test all edge cases on devnet
   - Consider professional audit for large deployments

## 🐛 Troubleshooting

### Common Issues:

1. **"Insufficient funds" error:**
   ```bash
   solana airdrop 2  # For devnet only
   ```

2. **"Program already in use" error:**
   - The vault is already initialized
   - Use the Python client's quick deposit test

3. **"Account does not exist" error:**
   - Ensure program is deployed
   - Verify you're on the correct network

4. **Build errors:**
   ```bash
   # Clean and rebuild
   cargo clean
   anchor build
   ```

## 📝 Environment Variables

Create a `.env` file for production:
```env
ANCHOR_WALLET=~/.config/solana/id.json
ANCHOR_PROVIDER_URL=https://api.mainnet-beta.solana.com
VAULT_PROGRAM_ID=YOUR_DEPLOYED_PROGRAM_ID
```

## 🔗 Useful Commands

```bash
# Check Solana CLI config
solana config get

# View program account
solana account YOUR_PROGRAM_ID

# Get transaction details
solana confirm -v TRANSACTION_SIGNATURE

# Monitor program in real-time
watch -n 2 'solana account YOUR_PROGRAM_ID --url devnet'
```

## 📚 Resources

- [Solana Explorer](https://explorer.solana.com/)
- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [anchorpy Documentation](https://kevinheavey.github.io/anchorpy/)

## Next Steps

1. ✅ Complete devnet testing
2. ✅ Verify all functionality works
3. ✅ Review security considerations
4. 🚀 Deploy to mainnet
5. 📊 Monitor and maintain

---

**Note:** Always test thoroughly on devnet before deploying to mainnet. Mainnet deployments cost real SOL and mistakes can be expensive.