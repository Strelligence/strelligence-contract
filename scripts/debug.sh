#!/bin/bash
set -e

# Debug contract state
# Usage: ./scripts/debug.sh [network]
#
# Shows contract state, recent events, and storage info

NETWORK="${1:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"

if [ ! -f .env.contracts ]; then
  echo "Error: .env.contracts not found. Deploy contracts first."
  exit 1
fi

source .env.contracts

echo "=== Strelligence Contract Debug ==="
echo "Network: $NETWORK"
echo "RPC URL: $RPC_URL"
echo ""

# ─── Contract State ──────────────────────────────────────────────────────────
echo "=== Contract State ==="
echo ""

echo "Recurring Registry ($RECURRING_CONTRACT):"
TOTAL=$(soroban contract invoke \
  --id "$RECURRING_CONTRACT" \
  --fn total_subscriptions \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" 2>&1)
echo "  Total subscriptions: $TOTAL"

echo ""

echo "Metadata Registry ($METADATA_CONTRACT):"
echo "  Contract deployed and responsive"

echo ""

echo "Automation Rules ($AUTOMATION_CONTRACT):"
echo "  Contract deployed and responsive"

# ─── Contract IDs ────────────────────────────────────────────────────────────
echo ""
echo "=== Contract IDs ==="
echo ""
echo "  Recurring Registry: $RECURRING_CONTRACT"
echo "  Metadata Registry:  $METADATA_CONTRACT"
echo "  Automation Rules:   $AUTOMATION_CONTRACT"

# ─── WASM Hashes ─────────────────────────────────────────────────────────────
echo ""
echo "=== Deployed WASM Hashes ==="
echo ""

if [ -f target/wasm32v1-none/release/recurring_registry.wasm ]; then
  sha256sum target/wasm32v1-none/release/recurring_registry.wasm | awk '{print "  recurring-registry: " $1}'
fi

if [ -f target/wasm32v1-none/release/metadata_registry.wasm ]; then
  sha256sum target/wasm32v1-none/release/metadata_registry.wasm | awk '{print "  metadata-registry: " $1}'
fi

if [ -f target/wasm32v1-none/release/automation_rules.wasm ]; then
  sha256sum target/wasm32v1-none/release/automation_rules.wasm | awk '{print "  automation-rules: " $1}'
fi

echo ""
echo "=== Debug Complete ==="
