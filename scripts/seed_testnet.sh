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
# Configuration
ENV_FILE=".env.testnet"

# Load environment variables
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: $ENV_FILE not found${NC}"
    echo "Please run deploy.sh first to deploy contracts and generate $ENV_FILE"
    exit 1
fi

source "$ENV_FILE"

# Check if contracts are deployed
if [ -z "$REGISTRY_CONTRACT_ID" ] || [ -z "$ESCROW_CONTRACT_ID" ] || [ -z "$SETTLEMENT_CONTRACT_ID" ]; then
    echo -e "${RED}Error: Contract IDs not set in $ENV_FILE${NC}"
    echo "Please run deploy.sh first to deploy contracts"
    exit 1
fi

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org:443"

echo -e "${GREEN}=== Stellar Testnet Seeding ===${NC}"
echo "Network: $NETWORK"
echo "RPC URL: $RPC_URL"
echo ""

# Function to invoke contract
invoke_contract() {
    local contract_id=$1
    local function_name=$2
    local args=$3
    
    soroban contract invoke \
        --id "$contract_id" \
        --fn "$function_name" \
        --source "$DEPLOYER_SECRET_KEY" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "Test SDF Network ; September 2015" \
        $args
}

# Function to read contract
read_contract() {
    local contract_id=$1
    local function_name=$2
    local args=$3
    
    soroban contract read \
        --id "$contract_id" \
        --fn "$function_name" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "Test SDF Network ; September 2015" \
        $args
}

# Generate test IDs
WAVE_PROGRAM_ID="test_program_$(date +%s)"
WAVE_ID="test_wave_$(date +%s)"
SETTLEMENT_ID="test_settlement_$(date +%s)"

echo -e "${YELLOW}Seeding test data...${NC}"
echo "Wave Program ID: $WAVE_PROGRAM_ID"
echo "Wave ID: $WAVE_ID"
echo "Settlement ID: $SETTLEMENT_ID"
echo ""

# Register a sample Wave Program
echo -e "${YELLOW}1. Registering Wave Program...${NC}"
invoke_contract "$REGISTRY_CONTRACT_ID" "register_program" \
    "--program_id $WAVE_PROGRAM_ID \
     --name 'Test Wave Program' \
     --description 'A test program for integration testing' \
     --creator $DEPLOYER_PUBLIC_KEY \
     --escrow_contract $ESCROW_CONTRACT_ID"
echo -e "${GREEN}✓ Wave Program registered${NC}"

# Verify program registration
echo "Verifying program registration..."
PROGRAM_DETAILS=$(read_contract "$REGISTRY_CONTRACT_ID" "get_program" "--program_id $WAVE_PROGRAM_ID")
echo "Program details: $PROGRAM_DETAILS"
echo ""

# Open a Wave
echo -e "${YELLOW}2. Opening Wave escrow...${NC}"
invoke_contract "$ESCROW_CONTRACT_ID" "open_wave" \
    "--wave_id $WAVE_ID \
     --program_id $WAVE_PROGRAM_ID \
     --creator $DEPLOYER_PUBLIC_KEY \
     --amount 10000000"
echo -e "${GREEN}✓ Wave opened${NC}"

# Verify wave creation
echo "Verifying wave creation..."
WAVE_DETAILS=$(read_contract "$ESCROW_CONTRACT_ID" "get_wave" "--wave_id $WAVE_ID")
echo "Wave details: $WAVE_DETAILS"
echo ""

# Fund the escrow
echo -e "${YELLOW}3. Funding Wave escrow...${NC}"
invoke_contract "$ESCROW_CONTRACT_ID" "fund_wave" \
    "--wave_id $WAVE_ID \
     --funder $DEPLOYER_PUBLIC_KEY \
     --amount 50000000"
echo -e "${GREEN}✓ Wave funded${NC}"

# Verify funding
echo "Verifying wave funding..."
WAVE_DETAILS=$(read_contract "$ESCROW_CONTRACT_ID" "get_wave" "--wave_id $WAVE_ID")
echo "Wave details after funding: $WAVE_DETAILS"
echo ""

# Create a settlement proposal
echo -e "${YELLOW}4. Creating settlement proposal...${NC}"
invoke_contract "$SETTLEMENT_CONTRACT_ID" "create_settlement" \
    "--settlement_id $SETTLEMENT_ID \
     --wave_id $WAVE_ID \
     --recipient $DEPLOYER_PUBLIC_KEY \
     --amount 30000000 \
     --proposer $DEPLOYER_PUBLIC_KEY"
echo -e "${GREEN}✓ Settlement proposal created${NC}"

# Verify settlement creation
echo "Verifying settlement creation..."
SETTLEMENT_DETAILS=$(read_contract "$SETTLEMENT_CONTRACT_ID" "get_settlement" "--settlement_id $SETTLEMENT_ID")
echo "Settlement details: $SETTLEMENT_DETAILS"
echo ""

# Approve settlement
echo -e "${YELLOW}5. Approving settlement...${NC}"
invoke_contract "$SETTLEMENT_CONTRACT_ID" "approve_settlement" "--settlement_id $SETTLEMENT_ID"
echo -e "${GREEN}✓ Settlement approved${NC}"

# Verify approval
echo "Verifying settlement approval..."
SETTLEMENT_DETAILS=$(read_contract "$SETTLEMENT_CONTRACT_ID" "get_settlement" "--settlement_id $SETTLEMENT_ID")
echo "Settlement details after approval: $SETTLEMENT_DETAILS"
echo ""

# Update .env.testnet with test IDs
echo -e "${YELLOW}Updating $ENV_FILE with test IDs...${NC}"
sed -i.bak "s/^WAVE_PROGRAM_ID=.*/WAVE_PROGRAM_ID=$WAVE_PROGRAM_ID/" "$ENV_FILE"
sed -i.bak "s/^WAVE_ID=.*/WAVE_ID=$WAVE_ID/" "$ENV_FILE"
rm -f "$ENV_FILE.bak"
echo -e "${GREEN}✓ Test IDs saved${NC}"

echo ""
echo -e "${GREEN}=== Seeding Complete ===${NC}"
echo ""
echo "Test Data Created:"
echo "  Wave Program ID: $WAVE_PROGRAM_ID"
echo "  Wave ID: $WAVE_ID"
echo "  Settlement ID: $SETTLEMENT_ID"
echo ""
echo -e "${GREEN}Seeding successful!${NC}"
echo "You can now use these IDs for integration testing."
