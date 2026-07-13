#!/bin/bash
set -e

# Query contract state
# Usage: ./scripts/query.sh <contract_id> <function> [args...]
#
# Examples:
#   ./scripts/query.sh CONTRACT_ID total_subscriptions
#   ./scripts/query.sh CONTRACT_ID get_subscription --arg id=1
#   ./scripts/query.sh CONTRACT_ID get_wallet_metadata --arg owner=G...

if [ $# -lt 2 ]; then
  echo "Usage: $0 <contract_id> <function> [args...]"
  echo ""
  echo "Examples:"
  echo "  $0 CONTRACT_ID total_subscriptions"
  echo "  $0 CONTRACT_ID get_subscription --arg id=1"
  echo "  $0 CONTRACT_ID get_wallet_metadata --arg owner=G..."
  exit 1
fi

CONTRACT=$1
FUNCTION=$2
shift 2

NETWORK="${NETWORK:-testnet}"
RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"

soroban contract invoke \
  --id "$CONTRACT" \
  --fn "$FUNCTION" \
  --network "$NETWORK" \
  --rpc-url "$RPC_URL" \
  -- "$@"
