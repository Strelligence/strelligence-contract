#!/bin/bash
set -e

NETWORK="mainnet"
RPC_URL="https://horizon.stellar.org"
HORIZON_URL="https://horizon.stellar.org"

DRY_RUN=false
if [ "$1" = "--dry-run" ]; then
  DRY_RUN=true
  echo "DRY RUN MODE — no contracts will be deployed"
  echo ""
fi

# ─── Safety Checks ───────────────────────────────────────────────────────────

if [ -z "$SOROBAN_SECRET" ]; then
  echo "Error: SOROBAN_SECRET environment variable is not set."
  echo "Export your secret key: export SOROBAN_SECRET=SA..."
  exit 1
fi

if [ ! -f target/wasm32v1-none/release/recurring_registry.wasm ] || \
   [ ! -f target/wasm32v1-none/release/metadata_registry.wasm ] || \
   [ ! -f target/wasm32v1-none/release/automation_rules.wasm ]; then
  echo "Error: WASM files not found. Run ./scripts/build.sh first."
  exit 1
fi

SECRET="$SOROBAN_SECRET"

# ─── Compute WASM hashes for verification ────────────────────────────────────
RECURRING_HASH=$(sha256sum target/wasm32v1-none/release/recurring_registry.wasm | awk '{print $1}')
METADATA_HASH=$(sha256sum target/wasm32v1-none/release/metadata_registry.wasm | awk '{print $1}')
AUTOMATION_HASH=$(sha256sum target/wasm32v1-none/release/automation_rules.wasm | awk '{print $1}')

echo "=== Strelligence Mainnet Deployment ==="
echo ""
echo "Network:    $NETWORK"
echo "RPC URL:    $RPC_URL"
echo ""
echo "WASM Hashes:"
echo "  Recurring Registry: $RECURRING_HASH"
echo "  Metadata Registry:  $METADATA_HASH"
echo "  Automation Rules:   $AUTOMATION_HASH"
echo ""

if [ "$DRY_RUN" = true ]; then
  echo "Dry run complete. No contracts deployed."
  exit 0
fi

# ─── Confirmation Prompt ─────────────────────────────────────────────────────
echo "WARNING: You are about to deploy to MAINNET."
echo "This action is irreversible and will spend real XLM."
echo ""
read -p "Type 'DEPLOY' to confirm: " CONFIRM
if [ "$CONFIRM" != "DEPLOY" ]; then
  echo "Deployment cancelled."
  exit 1
fi

echo ""
echo "Deploying to mainnet..."
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

# ─── Post-deployment verification ────────────────────────────────────────────
echo ""
echo "Verifying deployed contracts..."
echo ""

echo "Checking Recurring Registry..."
RECURRING_TOTAL=$(soroban contract invoke \
  --id "$RECURRING_CONTRACT" \
  --fn total_subscriptions \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET" 2>&1) && echo "  Recurring Registry: OK (total_subscriptions: $RECURRING_TOTAL)" || echo "  Recurring Registry: WARNING — could not verify"

echo "Checking Metadata Registry..."
METADATA_RESULT=$(soroban contract invoke \
  --id "$METADATA_CONTRACT" \
  --fn get_wallet_metadata \
  --arg owner="$SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET" 2>&1) && echo "  Metadata Registry:  OK" || echo "  Metadata Registry:  WARNING — could not verify"

echo "Checking Automation Rules..."
AUTOMATION_RESULT=$(soroban contract invoke \
  --id "$AUTOMATION_CONTRACT" \
  --fn list_wallet_rules \
  --arg owner="$SECRET" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET" 2>&1) && echo "  Automation Rules:   OK" || echo "  Automation Rules:   WARNING — could not verify"

echo ""
echo "=== Deployment Complete ==="
echo ""
echo "Contract IDs:"
echo "  Recurring Registry: $RECURRING_CONTRACT"
echo "  Metadata Registry:  $METADATA_CONTRACT"
echo "  Automation Rules:   $AUTOMATION_CONTRACT"
echo ""
echo "Contract IDs saved to .env.contracts"
