# Strelligence Event Indexer

Reference implementation of a Stellar event indexer for Strelligence contracts.

## Overview

This indexer listens for contract events and processes them in real-time.
It supports all event types from the three main contracts:

- **Recurring Registry:** subscription created/updated/cancelled/paused, payment confirmed
- **Metadata Registry:** metadata added/updated
- **Automation Rules:** rule created/updated/paused/deleted/executed

## Setup

```bash
cd examples/event-indexer
npm install
```

## Configuration

Set environment variables:

```bash
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"
export SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export RECURRING_CONTRACT="C..."
export METADATA_CONTRACT="C..."
export AUTOMATION_CONTRACT="C..."
export START_LEDGER=100000
export POLLING_INTERVAL_MS=5000
```

Or load from `.env.contracts`:

```bash
source ../../.env.contracts
```

## Running

```bash
# Development
npm run dev

# Production
npm run build
npm start
```

## Event Types

| Symbol | Contract | Description |
|--------|----------|-------------|
| `sub_crtd` | Recurring Registry | Subscription created |
| `sub_upd` | Recurring Registry | Subscription updated |
| `sub_can` | Recurring Registry | Subscription cancelled |
| `sub_psd` | Recurring Registry | Subscription paused |
| `pay_conf` | Recurring Registry | Payment confirmed |
| `meta_add` | Metadata Registry | Metadata added |
| `meta_upd` | Metadata Registry | Metadata updated |
| `rule_cr` | Automation Rules | Rule created |
| `rule_up` | Automation Rules | Rule updated |
| `rule_ps` | Automation Rules | Rule paused |
| `rule_dl` | Automation Rules | Rule deleted |
| `rule_ex` | Automation Rules | Rule executed |

## Extending

To add custom event handlers:

1. Create a new handler in `src/handlers/`
2. Import and register it in `src/index.ts`
3. Add the event symbol to `EVENT_HANDLERS`

## Error Handling

The indexer includes retry logic for network errors. If processing fails for
a specific ledger, it will retry on the next polling cycle.

## Architecture

```
┌─────────────────────────────────────────────┐
│           Event Indexer Process             │
│                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │Subscription│ │Metadata│  │  Rule   │    │
│  │ Handlers │  │Handlers│  │Handlers │    │
│  └────┬─────┘  └────┬────┘  └────┬────┘    │
│       │              │            │          │
│       └──────────────┼────────────┘          │
│                      │                       │
│              ┌───────▼───────┐               │
│              │  Event Router │               │
│              └───────┬───────┘               │
│                      │                       │
└──────────────────────┼───────────────────────┘
                       │
                       ▼
              Stellar Blockchain
```
