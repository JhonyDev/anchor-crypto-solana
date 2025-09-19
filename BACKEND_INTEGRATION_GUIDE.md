# Django Backend & React Frontend Integration Guide
## For Solana Multi-User Vault Program

### Program Overview
**Deployed Program ID**: `5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL`  
**Network**: Solana DevNet (ready for MainNet deployment)  
**Purpose**: Multi-user vault allowing independent deposits and withdrawals

---

## 🎯 What This Program Does

The deployed Anchor program is a **multi-user vault** that allows:
- Each user to have their own isolated account within the vault
- Users to deposit SOL independently
- Users to withdraw only their own funds
- Tracking of individual balances, deposits, and withdrawals
- Secure isolation between users (no cross-account access)

---

## 🏗️ Architecture Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  React Frontend │────▶│  Django Backend  │────▶│ Solana Program  │
│                 │     │                  │     │   (DevNet)      │
│  - Wallet       │     │  - API Routes   │     │                 │
│  - UI Forms     │     │  - Transaction   │     │  - Vault PDA    │
│  - Balance View │     │    Builder       │     │  - User PDAs    │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

---

## 📚 Key Concepts

### 1. Program Accounts

#### **Vault Account** (Global)
- **PDA Seeds**: `[b"vault"]`
- **Purpose**: Tracks global vault state
- **Fields**:
  - `authority`: Original initializer (admin)
  - `total_deposits`: Sum of all active deposits
  - `bump`: PDA bump seed

#### **Vault Funds PDA** (Global)
- **PDA Seeds**: `[b"vault_pda"]`
- **Purpose**: Holds all deposited SOL
- **Type**: System account (holds lamports)

#### **User Vault Account** (Per User)
- **PDA Seeds**: `[b"user_vault", user_public_key]`
- **Purpose**: Tracks individual user's vault activity
- **Fields**:
  ```rust
  {
    owner: PublicKey,           // User who owns this vault
    total_deposited: u64,       // Lifetime deposits
    total_withdrawn: u64,       // Lifetime withdrawals
    current_balance: u64,       // Available balance
    last_transaction: i64,      // Unix timestamp
    bump: u8                    // PDA bump seed
  }
  ```

---

## 🔧 Django Backend Implementation

### 1. Install Dependencies

```bash
pip install solana anchorpy django-cors-headers celery redis
```

### 2. Django Settings Configuration

```python
# settings.py

# Solana Configuration
SOLANA_NETWORK = 'devnet'  # or 'mainnet-beta' for production
SOLANA_RPC_URL = 'https://api.devnet.solana.com'
VAULT_PROGRAM_ID = '5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL'

# Add CORS settings for React frontend
INSTALLED_APPS = [
    # ...
    'corsheaders',
    'rest_framework',
]

MIDDLEWARE = [
    # ...
    'corsheaders.middleware.CorsMiddleware',
]

CORS_ALLOWED_ORIGINS = [
    "http://localhost:3000",  # React dev server
]
```

### 3. Vault Service Class

```python
# services/vault_service.py

import json
from solana.rpc.async_api import AsyncClient
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.system_program import SYS_PROGRAM_ID
from anchorpy import Program, Provider, Wallet, Context
from django.conf import settings
import base58

class VaultService:
    def __init__(self):
        self.client = AsyncClient(settings.SOLANA_RPC_URL)
        self.program_id = PublicKey(settings.VAULT_PROGRAM_ID)
        self.load_idl()
    
    def load_idl(self):
        # Load your IDL file
        with open('path/to/vault_app.json', 'r') as f:
            self.idl = json.load(f)
    
    @staticmethod
    def get_vault_pda():
        """Get the main vault PDA"""
        return PublicKey.find_program_address(
            [b"vault"],
            PublicKey(settings.VAULT_PROGRAM_ID)
        )
    
    @staticmethod
    def get_vault_funds_pda():
        """Get the vault funds PDA"""
        return PublicKey.find_program_address(
            [b"vault_pda"],
            PublicKey(settings.VAULT_PROGRAM_ID)
        )
    
    @staticmethod
    def get_user_vault_pda(user_pubkey: str):
        """Get a user's vault PDA"""
        user_pk = PublicKey(user_pubkey)
        return PublicKey.find_program_address(
            [b"user_vault", bytes(user_pk)],
            PublicKey(settings.VAULT_PROGRAM_ID)
        )
    
    async def get_user_balance(self, user_pubkey: str):
        """Get user's vault balance"""
        user_vault_pda, _ = self.get_user_vault_pda(user_pubkey)
        
        # Create a dummy wallet for read-only operations
        dummy_keypair = Keypair()
        wallet = Wallet(dummy_keypair)
        provider = Provider(self.client, wallet)
        program = Program(self.idl, self.program_id, provider)
        
        try:
            account = await program.account["UserVaultAccount"].fetch(user_vault_pda)
            return {
                "current_balance": account.current_balance / 1e9,  # Convert to SOL
                "total_deposited": account.total_deposited / 1e9,
                "total_withdrawn": account.total_withdrawn / 1e9,
                "last_transaction": account.last_transaction
            }
        except:
            return {
                "current_balance": 0,
                "total_deposited": 0,
                "total_withdrawn": 0,
                "last_transaction": None
            }
    
    async def build_deposit_transaction(self, user_pubkey: str, amount_sol: float):
        """Build a deposit transaction for the user to sign"""
        vault_pda, _ = self.get_vault_pda()
        vault_funds_pda, _ = self.get_vault_funds_pda()
        user_vault_pda, _ = self.get_user_vault_pda(user_pubkey)
        
        amount_lamports = int(amount_sol * 1e9)
        
        # This returns the transaction for the frontend to sign
        return {
            "instruction": "deposit",
            "accounts": {
                "vault": str(vault_pda),
                "vaultPda": str(vault_funds_pda),
                "userVault": str(user_vault_pda),
                "depositor": user_pubkey,
                "systemProgram": str(SYS_PROGRAM_ID)
            },
            "args": {
                "amount": amount_lamports
            }
        }
    
    async def build_withdraw_transaction(self, user_pubkey: str, amount_sol: float):
        """Build a withdrawal transaction for the user to sign"""
        vault_pda, _ = self.get_vault_pda()
        vault_funds_pda, _ = self.get_vault_funds_pda()
        user_vault_pda, _ = self.get_user_vault_pda(user_pubkey)
        
        amount_lamports = int(amount_sol * 1e9)
        
        return {
            "instruction": "withdraw",
            "accounts": {
                "vault": str(vault_pda),
                "vaultPda": str(vault_funds_pda),
                "userVault": str(user_vault_pda),
                "owner": user_pubkey,
                "recipient": user_pubkey,  # Can be different address
                "systemProgram": str(SYS_PROGRAM_ID)
            },
            "args": {
                "amount": amount_lamports
            }
        }
```

### 4. Django API Views

```python
# views.py

from rest_framework.views import APIView
from rest_framework.response import Response
from rest_framework import status
from .services.vault_service import VaultService
import asyncio

class UserBalanceView(APIView):
    """Get user's vault balance"""
    
    def get(self, request, user_pubkey):
        vault_service = VaultService()
        balance_data = asyncio.run(
            vault_service.get_user_balance(user_pubkey)
        )
        return Response(balance_data)

class BuildDepositView(APIView):
    """Build deposit transaction"""
    
    def post(self, request):
        user_pubkey = request.data.get('user_pubkey')
        amount_sol = request.data.get('amount')
        
        if not user_pubkey or not amount_sol:
            return Response(
                {"error": "Missing required fields"},
                status=status.HTTP_400_BAD_REQUEST
            )
        
        vault_service = VaultService()
        tx_data = asyncio.run(
            vault_service.build_deposit_transaction(user_pubkey, amount_sol)
        )
        return Response(tx_data)

class BuildWithdrawView(APIView):
    """Build withdrawal transaction"""
    
    def post(self, request):
        user_pubkey = request.data.get('user_pubkey')
        amount_sol = request.data.get('amount')
        
        vault_service = VaultService()
        tx_data = asyncio.run(
            vault_service.build_withdraw_transaction(user_pubkey, amount_sol)
        )
        return Response(tx_data)

class VaultStatsView(APIView):
    """Get overall vault statistics"""
    
    def get(self, request):
        # Implementation for vault stats
        pass
```

### 5. URL Configuration

```python
# urls.py

from django.urls import path
from . import views

urlpatterns = [
    path('api/vault/balance/<str:user_pubkey>/', views.UserBalanceView.as_view()),
    path('api/vault/deposit/', views.BuildDepositView.as_view()),
    path('api/vault/withdraw/', views.BuildWithdrawView.as_view()),
    path('api/vault/stats/', views.VaultStatsView.as_view()),
]
```

---

## ⚛️ React Frontend Implementation

### 1. Install Dependencies

```bash
npm install @solana/web3.js @solana/wallet-adapter-react \
  @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets \
  @coral-xyz/anchor axios
```

### 2. Wallet Provider Setup

```jsx
// App.jsx

import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import '@solana/wallet-adapter-react-ui/styles.css';

const network = WalletAdapterNetwork.Devnet;
const endpoint = clusterApiUrl(network);
const wallets = [new PhantomWalletAdapter()];

function App() {
  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <VaultApp />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
```

### 3. Vault Service Hook

```jsx
// hooks/useVault.js

import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Program, AnchorProvider, web3 } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import { useState, useEffect } from 'react';
import axios from 'axios';
import idl from './vault_app.json';

const PROGRAM_ID = new PublicKey('5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL');
const API_BASE = 'http://localhost:8000/api';

export const useVault = () => {
  const { connection } = useConnection();
  const { publicKey, signTransaction, sendTransaction } = useWallet();
  const [balance, setBalance] = useState(null);
  const [loading, setLoading] = useState(false);

  // Fetch user balance
  const fetchBalance = async () => {
    if (!publicKey) return;
    
    try {
      const response = await axios.get(
        `${API_BASE}/vault/balance/${publicKey.toString()}/`
      );
      setBalance(response.data);
    } catch (error) {
      console.error('Error fetching balance:', error);
    }
  };

  // Deposit funds
  const deposit = async (amount) => {
    if (!publicKey || !signTransaction) {
      alert('Please connect your wallet');
      return;
    }

    setLoading(true);
    try {
      // Get transaction data from backend
      const response = await axios.post(`${API_BASE}/vault/deposit/`, {
        user_pubkey: publicKey.toString(),
        amount: amount
      });

      const txData = response.data;

      // Create provider
      const provider = new AnchorProvider(
        connection,
        { publicKey, signTransaction },
        { commitment: 'confirmed' }
      );

      // Initialize program
      const program = new Program(idl, PROGRAM_ID, provider);

      // Execute transaction
      const tx = await program.methods
        .deposit(new web3.BN(txData.args.amount))
        .accounts({
          vault: new PublicKey(txData.accounts.vault),
          vaultPda: new PublicKey(txData.accounts.vaultPda),
          userVault: new PublicKey(txData.accounts.userVault),
          depositor: publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();

      console.log('Deposit successful:', tx);
      await fetchBalance(); // Refresh balance
      return tx;
    } catch (error) {
      console.error('Deposit error:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  // Withdraw funds
  const withdraw = async (amount) => {
    if (!publicKey || !signTransaction) {
      alert('Please connect your wallet');
      return;
    }

    setLoading(true);
    try {
      const response = await axios.post(`${API_BASE}/vault/withdraw/`, {
        user_pubkey: publicKey.toString(),
        amount: amount
      });

      const txData = response.data;

      const provider = new AnchorProvider(
        connection,
        { publicKey, signTransaction },
        { commitment: 'confirmed' }
      );

      const program = new Program(idl, PROGRAM_ID, provider);

      const tx = await program.methods
        .withdraw(new web3.BN(txData.args.amount))
        .accounts({
          vault: new PublicKey(txData.accounts.vault),
          vaultPda: new PublicKey(txData.accounts.vaultPda),
          userVault: new PublicKey(txData.accounts.userVault),
          owner: publicKey,
          recipient: publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();

      console.log('Withdrawal successful:', tx);
      await fetchBalance();
      return tx;
    } catch (error) {
      console.error('Withdrawal error:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (publicKey) {
      fetchBalance();
    }
  }, [publicKey]);

  return {
    balance,
    deposit,
    withdraw,
    loading,
    fetchBalance
  };
};
```

### 4. Vault Component

```jsx
// components/VaultInterface.jsx

import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { useVault } from '../hooks/useVault';

function VaultInterface() {
  const { publicKey } = useWallet();
  const { balance, deposit, withdraw, loading } = useVault();
  const [amount, setAmount] = useState('');
  const [activeTab, setActiveTab] = useState('deposit');

  const handleDeposit = async () => {
    try {
      const tx = await deposit(parseFloat(amount));
      alert(`Deposit successful! TX: ${tx}`);
      setAmount('');
    } catch (error) {
      alert(`Deposit failed: ${error.message}`);
    }
  };

  const handleWithdraw = async () => {
    try {
      const tx = await withdraw(parseFloat(amount));
      alert(`Withdrawal successful! TX: ${tx}`);
      setAmount('');
    } catch (error) {
      alert(`Withdrawal failed: ${error.message}`);
    }
  };

  return (
    <div className="vault-container">
      <h1>Multi-User Vault</h1>
      
      <div className="wallet-section">
        <WalletMultiButton />
      </div>

      {publicKey && (
        <>
          <div className="balance-section">
            <h2>Your Vault Balance</h2>
            {balance ? (
              <div>
                <p>Current Balance: {balance.current_balance} SOL</p>
                <p>Total Deposited: {balance.total_deposited} SOL</p>
                <p>Total Withdrawn: {balance.total_withdrawn} SOL</p>
              </div>
            ) : (
              <p>Loading balance...</p>
            )}
          </div>

          <div className="action-section">
            <div className="tabs">
              <button 
                className={activeTab === 'deposit' ? 'active' : ''}
                onClick={() => setActiveTab('deposit')}
              >
                Deposit
              </button>
              <button 
                className={activeTab === 'withdraw' ? 'active' : ''}
                onClick={() => setActiveTab('withdraw')}
              >
                Withdraw
              </button>
            </div>

            <div className="action-form">
              <input
                type="number"
                placeholder="Amount in SOL"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                step="0.01"
                min="0"
              />
              
              {activeTab === 'deposit' ? (
                <button 
                  onClick={handleDeposit} 
                  disabled={loading || !amount}
                >
                  {loading ? 'Processing...' : 'Deposit SOL'}
                </button>
              ) : (
                <button 
                  onClick={handleWithdraw} 
                  disabled={loading || !amount || amount > balance?.current_balance}
                >
                  {loading ? 'Processing...' : 'Withdraw SOL'}
                </button>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default VaultInterface;
```

---

## 🔑 Key Implementation Points

### Security Considerations

1. **Never store private keys in backend** - All signing happens on frontend
2. **Validate amounts** - Check for negative values, overflow
3. **Rate limiting** - Implement API rate limits
4. **CSRF protection** - Django CSRF tokens for POST requests
5. **Input sanitization** - Validate all public keys and amounts

### Transaction Flow

1. **User initiates action** → Frontend sends request to Django
2. **Django builds transaction** → Returns unsigned transaction data
3. **Frontend signs transaction** → Using connected wallet
4. **Frontend submits to blockchain** → Direct to Solana
5. **Backend verifies completion** → Optional webhook or polling

### Error Handling

```python
# Django error handling
class VaultError(Exception):
    pass

class InsufficientBalanceError(VaultError):
    pass

class InvalidPublicKeyError(VaultError):
    pass

# Check balance before withdrawal
user_balance = await get_user_balance(user_pubkey)
if user_balance['current_balance'] < amount:
    raise InsufficientBalanceError("Insufficient vault balance")
```

```jsx
// React error handling
try {
  await withdraw(amount);
} catch (error) {
  if (error.message.includes('InsufficientUserBalance')) {
    alert('You don\'t have enough balance in your vault');
  } else if (error.message.includes('User rejected')) {
    alert('Transaction cancelled');
  } else {
    alert(`Error: ${error.message}`);
  }
}
```

---

## 📊 Database Schema (Optional)

Track transactions in Django for analytics:

```python
# models.py

class VaultTransaction(models.Model):
    TRANSACTION_TYPES = [
        ('DEPOSIT', 'Deposit'),
        ('WITHDRAW', 'Withdrawal'),
    ]
    
    user_pubkey = models.CharField(max_length=44)
    transaction_type = models.CharField(max_length=10, choices=TRANSACTION_TYPES)
    amount_sol = models.DecimalField(max_digits=20, decimal_places=9)
    signature = models.CharField(max_length=88, unique=True)
    status = models.CharField(max_length=20, default='pending')
    created_at = models.DateTimeField(auto_now_add=True)
    confirmed_at = models.DateTimeField(null=True, blank=True)
    
    class Meta:
        ordering = ['-created_at']
```

---

## 🚀 Deployment Checklist

### Backend
- [ ] Set up environment variables for RPC URLs
- [ ] Configure CORS for production domain
- [ ] Set up Redis for caching user balances
- [ ] Implement webhook for transaction confirmations
- [ ] Add comprehensive logging
- [ ] Set up monitoring (e.g., Sentry)

### Frontend
- [ ] Build production bundle
- [ ] Configure correct network (mainnet-beta for production)
- [ ] Add loading states and error boundaries
- [ ] Implement transaction history view
- [ ] Add balance refresh button
- [ ] Test with multiple wallets

### Smart Contract
- [ ] Audit the program code
- [ ] Deploy to mainnet-beta
- [ ] Transfer upgrade authority to multisig
- [ ] Document all PDAs and accounts

---

## 📚 Additional Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Web3.js](https://solana-labs.github.io/solana-web3.js/)
- [Wallet Adapter](https://github.com/solana-labs/wallet-adapter)
- [Program Explorer](https://explorer.solana.com/address/5rLtuZQcfq1Cjs2R9aAmGoURLwm7S6NDQbUVA94jDKFL?cluster=devnet)

---

## 🆘 Common Issues & Solutions

### Issue: "Account does not exist"
**Solution**: User hasn't deposited yet. The account is created on first deposit.

### Issue: "Insufficient SOL for fees"
**Solution**: User needs ~0.002 SOL for transaction fees.

### Issue: "Transaction simulation failed"
**Solution**: Check account balances and ensure PDAs are derived correctly.

### Issue: "Blockhash not found"
**Solution**: Transaction took too long. Implement retry logic with fresh blockhash.

---

This guide provides everything needed to integrate the multi-user vault into your Django/React application. The program handles all the complexity of user isolation and balance tracking on-chain, while your backend serves as the interface builder and your frontend handles wallet interactions.