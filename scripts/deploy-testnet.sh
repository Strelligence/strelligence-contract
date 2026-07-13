#!/bin/bash
set -e

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"

if [ -z "$SOROBAN_SECRET" ]; then
  echo "Error: SOROBAN_SECRET environment variable is not set."
  echo "Export your secret key: export SOROBAN_SECRET=SA..."
  exit 1
fi

SECRET="$SOROBAN_SECRET"

echo "Deploying Strelligence contracts to testnet..."
echo ""

# ─── Deploy Recurring Registry ───────────────────────────────────────────────
echo "Deploying Recurring Registry..."
RECURRING_ID=$(soroban contract deploy \
  --wasm target/wasm32v1-none/release/recurring_registry.wasm \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET")
echo "  Recurring Registry: $RECURRING_ID"

# ─── Deploy Metadata Registry ────────────────────────────────────────────────
echo "Deploying Metadata Registry..."
METADATA_ID=$(soroban contract deploy \
  --wasm target/wasm32v1-none/release/metadata_registry.wasm \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET")
echo "  Metadata Registry:  $METADATA_ID"

# ─── Deploy Automation Rules ─────────────────────────────────────────────────
echo "Deploying Automation Rules..."
AUTOMATION_ID=$(soroban contract deploy \
  --wasm target/wasm32v1-none/release/automation_rules.wasm \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET")
echo "  Automation Rules:   $AUTOMATION_ID"

# ─── Save contract IDs ───────────────────────────────────────────────────────
echo ""
echo "RECURRING_CONTRACT=$RECURRING_ID" > .env.contracts
echo "METADATA_CONTRACT=$METADATA_ID" >> .env.contracts
echo "AUTOMATION_CONTRACT=$AUTOMATION_ID" >> .env.contracts

echo ""
echo "Deployed! Contract IDs saved to .env.contracts"
echo ""
echo "Next step: run ./scripts/initialize.sh to set up contract state."
