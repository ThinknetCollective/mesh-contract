#!/bin/bash

set -e

# Stellar Testnet Seeding Script
# Registers a sample Wave Program, opens a Wave, and funds the escrow
# Requires contracts to be deployed first (run deploy.sh)

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org:443"
SOURCE_WALLET=".deploy/wallet.json"
ENV_FILE=".env.testnet"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Stellar Testnet Seeding ===${NC}"

# Check if .env.testnet exists
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: $ENV_FILE not found. Please run deploy.sh first.${NC}"
    exit 1
fi

# Source environment variables
source "$ENV_FILE"

# Check if all required variables are set
if [ -z "$REGISTRY_CONTRACT_ID" ] || [ -z "$ESCROW_CONTRACT_ID" ] || [ -z "$SETTLEMENT_CONTRACT_ID" ]; then
    echo -e "${RED}Error: Missing contract IDs in $ENV_FILE${NC}"
    exit 1
fi

echo -e "${YELLOW}Registry: $REGISTRY_CONTRACT_ID${NC}"
echo -e "${YELLOW}Escrow: $ESCROW_CONTRACT_ID${NC}"
echo -e "${YELLOW}Settlement: $SETTLEMENT_CONTRACT_ID${NC}"

# Get public key
PUBLIC_KEY=$(stellar keys address --network "$NETWORK" --rpc-url "$RPC_URL" --source "$SOURCE_WALLET")
echo -e "${GREEN}Deployer Address: $PUBLIC_KEY${NC}"

# Fund account via Friendbot if needed
echo -e "${YELLOW}Ensuring account is funded...${NC}"
curl -X POST "https://friendbot.stellar.org/?addr=$PUBLIC_KEY" || echo "Account already funded"

# Sample Wave Program data
PROGRAM_ID="wave_program_001"
PROGRAM_METADATA="Sample Wave Program for integration testing"
FUNDING_TARGET=10000000  # 10 million stroops (1 XLM)

# Register a sample Wave Program
echo -e "${YELLOW}Registering sample Wave Program...${NC}"
soroban contract invoke \
    --id "$REGISTRY_CONTRACT_ID" \
    --source "$SOURCE_WALLET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    -- register_program \
    --program_id "$PROGRAM_ID" \
    --creator "$PUBLIC_KEY" \
    --metadata "$PROGRAM_METADATA" \
    --funding_target "$FUNDING_TARGET"

echo -e "${GREEN}Wave Program registered: $PROGRAM_ID${NC}"

# Open a Wave (escrow)
WAVE_ID="wave_001"
WAVE_AMOUNT=5000000  # 5 million stroops (0.5 XLM)

echo -e "${YELLOW}Opening Wave (escrow)...${NC}"
soroban contract invoke \
    --id "$ESCROW_CONTRACT_ID" \
    --source "$SOURCE_WALLET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    -- open_wave \
    --wave_id "$WAVE_ID" \
    --program_id "$PROGRAM_ID" \
    --funder "$PUBLIC_KEY" \
    --amount "$WAVE_AMOUNT"

echo -e "${GREEN}Wave opened: $WAVE_ID${NC}"

# Fund the escrow
echo -e "${YELLOW}Funding escrow...${NC}"
soroban contract invoke \
    --id "$ESCROW_CONTRACT_ID" \
    --source "$SOURCE_WALLET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    -- fund_wave \
    --wave_id "$WAVE_ID" \
    --funder "$PUBLIC_KEY" \
    --amount "$WAVE_AMOUNT"

echo -e "${GREEN}Escrow funded with $WAVE_AMOUNT stroops${NC}"

# Verify the data
echo -e "${YELLOW}Verifying Wave Program...${NC}"
soroban contract invoke \
    --id "$REGISTRY_CONTRACT_ID" \
    --source "$SOURCE_WALLET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    -- get_program \
    --program_id "$PROGRAM_ID"

echo -e "${YELLOW}Verifying Wave...${NC}"
soroban contract invoke \
    --id "$ESCROW_CONTRACT_ID" \
    --source "$SOURCE_WALLET" \
    --network "$NETWORK" \
    --rpc-url "$RPC_URL" \
    -- get_wave \
    --wave_id "$WAVE_ID"

echo -e "${GREEN}=== Testnet Seeding Complete ===${NC}"
echo -e "${GREEN}Program ID: $PROGRAM_ID${NC}"
echo -e "${GREEN}Wave ID: $WAVE_ID${NC}"
echo -e "${GREEN}Funding Amount: $WAVE_AMOUNT stroops${NC}"
