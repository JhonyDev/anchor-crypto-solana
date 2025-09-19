#!/bin/bash

# Devnet Deployment and Testing Script for Vault App
# This script handles the complete devnet deployment workflow

set -e  # Exit on error

echo "================================================"
echo "Vault App - Devnet Deployment Script"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Check Solana CLI configuration
echo -e "\n${YELLOW}Step 1: Checking Solana CLI configuration...${NC}"
solana config get

# Step 2: Switch to devnet
echo -e "\n${YELLOW}Step 2: Switching to devnet...${NC}"
solana config set --url https://api.devnet.solana.com
echo -e "${GREEN}✓ Switched to devnet${NC}"

# Step 3: Check wallet balance
echo -e "\n${YELLOW}Step 3: Checking wallet balance...${NC}"
BALANCE=$(solana balance)
echo "Current balance: $BALANCE"

# Check if balance is low
if [[ $(echo "$BALANCE" | awk '{print $1}' | awk -F. '{print $1}') -lt 2 ]]; then
    echo -e "${YELLOW}Balance is low. Requesting airdrop...${NC}"
    solana airdrop 2
    sleep 5  # Wait for airdrop to process
    echo -e "${GREEN}✓ Airdrop complete${NC}"
    solana balance
fi

# Step 4: Build the program
echo -e "\n${YELLOW}Step 4: Building the program...${NC}"
anchor build
echo -e "${GREEN}✓ Program built successfully${NC}"

# Step 5: Get the program ID
echo -e "\n${YELLOW}Step 5: Getting program ID...${NC}"
PROGRAM_ID=$(solana address -k target/deploy/vault_app-keypair.json)
echo "Program ID: $PROGRAM_ID"

# Step 6: Update program ID in lib.rs if needed
echo -e "\n${YELLOW}Step 6: Updating program ID in lib.rs...${NC}"
sed -i "s/declare_id!(\".*\")/declare_id!(\"$PROGRAM_ID\")/" programs/mytestproject/src/lib.rs
echo -e "${GREEN}✓ Program ID updated${NC}"

# Step 7: Update Anchor.toml with new program ID
echo -e "\n${YELLOW}Step 7: Updating Anchor.toml...${NC}"
sed -i "s/vault_app = \".*\"/vault_app = \"$PROGRAM_ID\"/" Anchor.toml
echo -e "${GREEN}✓ Anchor.toml updated${NC}"

# Step 8: Rebuild with correct program ID
echo -e "\n${YELLOW}Step 8: Rebuilding with correct program ID...${NC}"
anchor build
echo -e "${GREEN}✓ Program rebuilt with correct ID${NC}"

# Step 9: Deploy to devnet
echo -e "\n${YELLOW}Step 9: Deploying to devnet...${NC}"
anchor deploy --provider.cluster devnet
echo -e "${GREEN}✓ Program deployed to devnet!${NC}"

# Step 10: Display deployment info
echo -e "\n${GREEN}================================================${NC}"
echo -e "${GREEN}DEPLOYMENT SUCCESSFUL!${NC}"
echo -e "${GREEN}================================================${NC}"
echo -e "Program ID: ${YELLOW}$PROGRAM_ID${NC}"
echo -e "Network: ${YELLOW}Devnet${NC}"
echo -e "Explorer: ${YELLOW}https://explorer.solana.com/address/$PROGRAM_ID?cluster=devnet${NC}"
echo -e "\nIDL Location: ${YELLOW}target/idl/vault_app.json${NC}"
echo -e "\nNext steps:"
echo -e "1. Run the TypeScript tests: ${YELLOW}npm test${NC}"
echo -e "2. Run the Python client: ${YELLOW}python client_devnet.py${NC}"
echo -e "3. Check the explorer link above to see your program on-chain"
echo -e "${GREEN}================================================${NC}"