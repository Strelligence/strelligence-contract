#!/bin/bash
set -e

# Test contract interactions
# Usage: ./scripts/test-interaction.sh [network]
#
# Requires: SOROBAN_SECRET environment variable

NETWORK="${1:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"

if [ -z "$SOROBAN_SECRET" ]; then
  echo "Error: SOROBAN_SECRET not set"
  exit 1
fi

if [ ! -f .env.contracts ]; then
  echo "Error: .env.contracts not found. Deploy contracts first."
  exit 1
fi

source .env.contracts

echo "Testing contract interactions on $NETWORK..."
echo ""

# ─── Test Recurring Registry ─────────────────────────────────────────────────
echo "=== Recurring Registry ==="
echo ""

echo "Creating subscription..."
SUB_ID=$(soroban contract invoke \
  --id "$RECURRING_CONTRACT" \
  --fn create_subscription \
  --arg caller="$SOROBAN_SECRET" \
  --arg owner="$SOROBAN_SECRET" \
  --arg merchant="Netflix" \
  --arg merchant_address=null \
  --arg frequency=Monthly \
  --arg subscription_type=Subscription \
  --arg asset_code="USDC" \
  --arg asset_issuer="issuer1" \
  --arg amount=15000000 \
  --arg next_payment_ledger=100000 \
  --arg auto_detected=true \
  --arg custom_label=null \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SOROBAN_SECRET" 2>&1)
echo "  Subscription ID: $SUB_ID"

echo "Getting subscription..."
soroban contract invoke \
  --id "$RECURRING_CONTRACT" \
  --fn get_subscription \
  --arg id="$SUB_ID" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" 2>&1 | head -5
echo "  ..."

echo ""

# ─── Test Metadata Registry ──────────────────────────────────────────────────
echo "=== Metadata Registry ==="
echo ""

echo "Adding metadata..."
soroban contract invoke \
  --id "$METADATA_CONTRACT" \
  --fn add_metadata \
  --arg caller="$SOROBAN_SECRET" \
  --arg tx_hash="test_tx_001" \
  --arg category=Subscription \
  --arg sentiment=Negative \
  --arg tags='["streaming"]' \
  --arg label="Netflix Payment" \
  --arg notes="Test metadata" \
  --arg counterparty_label="Netflix" \
  --arg is_recurring=true \
  --arg recurring_id="$SUB_ID" \
  --arg ai_confidence=95 \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SOROBAN_SECRET" 2>&1
echo "  Metadata added"

echo "Getting metadata..."
soroban contract invoke \
  --id "$METADATA_CONTRACT" \
  --fn get_metadata \
  --arg tx_hash="test_tx_001" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" 2>&1 | head -5
echo "  ..."

echo ""

# ─── Test Automation Rules ───────────────────────────────────────────────────
echo "=== Automation Rules ==="
echo ""

echo "Creating rule..."
RULE_ID=$(soroban contract invoke \
  --id "$AUTOMATION_CONTRACT" \
  --fn create_rule \
  --arg caller="$SOROBAN_SECRET" \
  --arg rule_type=AutoSave \
  --arg trigger=OnIncomingPayment \
  --arg label="Auto-save 10%" \
  --arg trigger_params='{"percentage":10}' \
  --arg action_params='{"dest":"G..."}' \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  --source "$SOROBAN_SECRET" 2>&1)
echo "  Rule ID: $RULE_ID"

echo "Getting rule..."
soroban contract invoke \
  --id "$AUTOMATION_CONTRACT" \
  --fn get_rule \
  --arg id="$RULE_ID" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" 2>&1 | head -5
echo "  ..."

echo ""
echo "All tests passed!"
