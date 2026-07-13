# Strelligence Smart Contracts

Soroban smart contracts powering the programmable financial metadata layer for Strelligence.

## Overview

The Strelligence contracts provide:
- Recurring payment metadata tracking
- Transaction tagging and categorization
- Financial automation primitives
- Programmable treasury utilities

## Purpose

The contracts extend Stellar/Soroban transactions with:
- Structured metadata attached to on-chain transactions
- Recurring payment detection and tracking standards
- Programmable financial coordination via automation rules

## Tech Stack

- Rust (edition 2021)
- Soroban SDK v25.3.1
- Soroban CLI

## Contracts

### Recurring Registry
Tracks recurring payment relationships. Stores subscription records with merchant,
frequency, amount, and lifecycle state (Active → Paused → Cancelled).

### Metadata Registry
Stores standardized financial metadata. Attaches AI-classified categories,
sentiment, tags, and notes to individual transaction hashes.

### Automation Rules
Supports programmable financial routing and treasury logic. Registers rules
with triggers and actions that the backend executes.

## Building

```bash
# Build all contracts for WASM
./scripts/build.sh

# Or build individually
cargo build --target wasm32v1-none --release -p recurring-registry
cargo build --target wasm32v1-none --release -p metadata-registry
cargo build --target wasm32v1-none --release -p automation-rules
```

## WASM Binary Sizes

Optimized with `opt-level = "z"`, LTO, and symbol stripping:

| Contract | Size |
|----------|------|
| recurring-registry | ~30 KB |
| metadata-registry | ~20 KB |
| automation-rules | ~24 KB |

## Testing

```bash
# Run all tests (unit + integration)
cargo test --workspace

# Run individual contract tests
cargo test -p recurring-registry
cargo test -p metadata-registry
cargo test -p automation-rules

# Run integration tests
cargo test -p integration-tests
```

## Deployment

```bash
# Testnet
export SOROBAN_SECRET=SA...
./scripts/build.sh
./scripts/deploy-testnet.sh
./scripts/initialize.sh

# Mainnet (with safety checks)
./scripts/deploy-mainnet.sh
./scripts/deploy-mainnet.sh --dry-run
```

## Security

See [SECURITY.md](SECURITY.md) for the contract security audit checklist.

## Events

See [docs/events.md](docs/events.md) for the complete events reference with
parsing examples in JavaScript and Rust.

## Optimization Techniques

The WASM binaries are optimized using the following techniques configured in
`Cargo.toml`:

- **`opt-level = "z"`** — Minimize binary size aggressively
- **`lto = true`** — Link-Time Optimization across all crates
- **`codegen-units = 1`** — Single codegen unit for maximum optimization
- **`strip = true`** — Strip debug symbols from release binaries

These techniques reduce the WASM binary sizes by approximately 60–70% compared
to default debug builds while preserving all functionality (all 69 tests pass).
