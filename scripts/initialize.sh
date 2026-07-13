#!/bin/bash
set -e

NETWORK="testnet"
RPC_URL="https://soroban-testnet.stellar.org"

if [ -z "$SOROBAN_SECRET" ]; then
  echo "Error: SOROBAN_SECRET environment variable is not set."
  exit 1
fi

if [ ! -f .env.contracts ]; then
  echo "Error: .env.contracts not found. Run ./scripts/deploy-testnet.sh first."
  exit 1
fi

source .env.contracts

SECRET="$SOROBAN_SECRET"

echo "Initializing Strelligence contracts on testnet..."
echo ""

# ─── Verify contracts are responsive ─────────────────────────────────────────
echo "Verifying Recurring Registry..."
RECURRING_TOTAL=$(soroban contract invoke \
  --id "$RECURRING_CONTRACT" \
  --fn total_subscriptions \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SECRET")
echo "  Recurring Registry total_subscriptions: $RECURRING_TOTAL"

echo ""
echo "Initialization complete!"
echo ""
echo "Contract IDs:"
echo "  Recurring Registry: $RECURRING_CONTRACT"
echo "  Metadata Registry:  $METADATA_CONTRACT"
echo "  Automation Rules:   $AUTOMATION_CONTRACT"
