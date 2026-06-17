# Stellar Testnet Seeding Script (PowerShell)
# Registers a sample Wave Program, opens a Wave, and funds the escrow
# Requires contracts to be deployed first (run deploy.ps1)

$ErrorActionPreference = "Stop"

$NETWORK = "testnet"
$RPC_URL = "https://soroban-testnet.stellar.org:443"
$SOURCE_WALLET = ".deploy\wallet.json"
$ENV_FILE = ".env.testnet"

Write-Host "=== Stellar Testnet Seeding ===" -ForegroundColor Green

# Check if .env.testnet exists
if (-not (Test-Path $ENV_FILE)) {
    Write-Host "Error: $ENV_FILE not found. Please run deploy.ps1 first." -ForegroundColor Red
    exit 1
}

# Source environment variables
$envVars = Get-Content $ENV_FILE | Where-Object { $_ -match "=" } | ForEach-Object {
    $key, $value = $_.split("=", 2)
    [PSCustomObject]@{ Key = $key; Value = $value }
}

foreach ($var in $envVars) {
    Set-Item -Path "env:$($var.Key)" -Value $var.Value
}

# Check if all required variables are set
if (-not $env:REGISTRY_CONTRACT_ID -or -not $env:ESCROW_CONTRACT_ID -or -not $env:SETTLEMENT_CONTRACT_ID) {
    Write-Host "Error: Missing contract IDs in $ENV_FILE" -ForegroundColor Red
    exit 1
}

Write-Host "Registry: $env:REGISTRY_CONTRACT_ID" -ForegroundColor Yellow
Write-Host "Escrow: $env:ESCROW_CONTRACT_ID" -ForegroundColor Yellow
Write-Host "Settlement: $env:SETTLEMENT_CONTRACT_ID" -ForegroundColor Yellow

# Get public key
$PUBLIC_KEY = stellar keys address --network $NETWORK --rpc-url $RPC_URL --source $SOURCE_WALLET
Write-Host "Deployer Address: $PUBLIC_KEY" -ForegroundColor Green

# Fund account via Friendbot if needed
Write-Host "Ensuring account is funded..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri "https://friendbot.stellar.org/?addr=$PUBLIC_KEY" -Method Post | Out-Null
} catch {
    Write-Host "Account already funded" -ForegroundColor Yellow
}

# Sample Wave Program data
$PROGRAM_ID = "wave_program_001"
$PROGRAM_METADATA = "Sample Wave Program for integration testing"
$FUNDING_TARGET = 10000000  # 10 million stroops (1 XLM)

# Register a sample Wave Program
Write-Host "Registering sample Wave Program..." -ForegroundColor Yellow
soroban contract invoke --id $env:REGISTRY_CONTRACT_ID --source $SOURCE_WALLET --network $NETWORK --rpc-url $RPC_URL -- register_program --program_id $PROGRAM_ID --creator $PUBLIC_KEY --metadata $PROGRAM_METADATA --funding_target $FUNDING_TARGET
Write-Host "Wave Program registered: $PROGRAM_ID" -ForegroundColor Green

# Open a Wave (escrow)
$WAVE_ID = "wave_001"
$WAVE_AMOUNT = 5000000  # 5 million stroops (0.5 XLM)

Write-Host "Opening Wave (escrow)..." -ForegroundColor Yellow
soroban contract invoke --id $env:ESCROW_CONTRACT_ID --source $SOURCE_WALLET --network $NETWORK --rpc-url $RPC_URL -- open_wave --wave_id $WAVE_ID --program_id $PROGRAM_ID --funder $PUBLIC_KEY --amount $WAVE_AMOUNT
Write-Host "Wave opened: $WAVE_ID" -ForegroundColor Green

# Fund the escrow
Write-Host "Funding escrow..." -ForegroundColor Yellow
soroban contract invoke --id $env:ESCROW_CONTRACT_ID --source $SOURCE_WALLET --network $NETWORK --rpc-url $RPC_URL -- fund_wave --wave_id $WAVE_ID --funder $PUBLIC_KEY --amount $WAVE_AMOUNT
Write-Host "Escrow funded with $WAVE_AMOUNT stroops" -ForegroundColor Green

# Verify the data
Write-Host "Verifying Wave Program..." -ForegroundColor Yellow
soroban contract invoke --id $env:REGISTRY_CONTRACT_ID --source $SOURCE_WALLET --network $NETWORK --rpc-url $RPC_URL -- get_program --program_id $PROGRAM_ID

Write-Host "Verifying Wave..." -ForegroundColor Yellow
soroban contract invoke --id $env:ESCROW_CONTRACT_ID --source $SOURCE_WALLET --network $NETWORK --rpc-url $RPC_URL -- get_wave --wave_id $WAVE_ID

Write-Host "=== Testnet Seeding Complete ===" -ForegroundColor Green
Write-Host "Program ID: $PROGRAM_ID" -ForegroundColor Green
Write-Host "Wave ID: $WAVE_ID" -ForegroundColor Green
Write-Host "Funding Amount: $WAVE_AMOUNT stroops" -ForegroundColor Green
