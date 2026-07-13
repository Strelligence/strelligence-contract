# Integration Guide

How to interact with Strelligence contracts from a backend service.

## Contract Interaction Patterns

### Calling from Backend

All write operations require the caller's auth signature. Use the Soroban SDK
to build and sign transactions:

```rust
use soroban_sdk::{Address, Env, String};

// Initialize environment
let env = Env::default();
let contract_id = Address::from_str(&env, "CONTRACT_ADDRESS");

// Create a client
let client = RecurringRegistryContractClient::new(&env, &contract_id);

// Call a write function (requires auth)
client.create_subscription(
    &wallet_address,
    &wallet_address,
    &String::from_str(&env, "Netflix"),
    &None,
    &Frequency::Monthly,
    &SubscriptionType::Subscription,
    &String::from_str(&env, "USDC"),
    &String::from_str(&env, "issuer1"),
    &15_000_000,
    &1000,
    &true,
    &None,
);
```

### Reading State

Read functions do not require auth and can be called freely:

```rust
// Get a single record
let sub = client.get_subscription(&id);

// List all records for a wallet
let ids = client.list_wallet_subscriptions(&wallet);

// Filter by category
let expenses = metadata_client.get_metadata_by_category(
    &wallet,
    &TransactionCategory::Expense,
);
```

## Event Handling

Contracts emit events on every state change. Use the Stellar SDK to listen
for events:

```javascript
import { SorobanRpc } from "@stellar/stellar-sdk";

const server = new SorobanRpc.Server("https://soroban-testnet.stellar.org");

const events = await server.getEvents({
  contractId: RECURRING_CONTRACT_ID,
  startLedger: lastProcessedLedger,
  limit: 100,
});

for (const event of events.events) {
  const [topic0, owner] = event.topics;
  const eventType = topic0.value().toString();

  switch (eventType) {
    case "sub_crtd":
      await saveSubscription(event.data.value(), owner);
      break;
    case "meta_add":
      await saveMetadata(event.data.value(), owner);
      break;
    case "rule_ex":
      await recordExecution(event.data.value(), owner);
      break;
  }
}
```

## Common Patterns

### 1. Subscription + Metadata Linking

When a subscription payment is detected:

1. Create subscription record
2. Attach metadata to the transaction hash
3. Link metadata to subscription via `recurring_id`

```rust
// 1. Create subscription
let sub_id = recurring_client.create_subscription(...);

// 2. Attach metadata
metadata_client.add_metadata(
    &owner,
    &tx_hash,
    &TransactionCategory::Subscription,
    &TransactionSentiment::Negative,
    &tags,
    &label,
    &notes,
    &counterparty,
    &true,
    &Some(sub_id),  // Link to subscription
    &confidence,
);

// 3. Confirm payment
recurring_client.confirm_payment(&owner, &sub_id, &payment_ledger, &next_ledger);
```

### 2. Automation Rule Execution

When a rule triggers:

1. Backend detects trigger condition
2. Backend executes the action
3. Backend records execution on-chain

```rust
// 1. Backend detects trigger (off-chain)
// 2. Backend executes action (off-chain)

// 3. Record execution on-chain
automation_client.record_execution(&backend_address, &rule_id);
```

### 3. Batch Operations

For multiple records, create them in a single transaction:

```rust
for merchant in merchants {
    recurring_client.create_subscription(
        &owner,
        &owner,
        &String::from_str(&env, merchant),
        &None,
        &Frequency::Monthly,
        &SubscriptionType::Subscription,
        &String::from_str(&env, "USDC"),
        &String::from_str(&env, "issuer1"),
        &5_000_000,
        &1000,
        &true,
        &None,
    );
}
```

## Anti-Patterns

### Don't: Skip Auth Checks

Never call write functions without proper auth. The contract will reject
transactions that lack the required authorization.

### Don't: Store Large Data On-Chain

Keep metadata strings concise. Large payloads increase transaction costs.
Store detailed data off-chain and reference it via hash.

### Don't: Ignore Error Returns

Always handle `Result` types. Contract errors indicate invalid state
transitions that should be handled gracefully.

### Don't: Poll for State Changes

Use events instead of polling. Events are pushed to subscribers and are
more efficient than repeated read calls.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Backend Service                      │
│                                                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Payment  │  │   AI     │  │  Rule    │              │
│  │ Detector │  │Classifier│  │ Executor │              │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘              │
│       │              │              │                    │
└───────┼──────────────┼──────────────┼────────────────────┘
        │              │              │
        ▼              ▼              ▼
┌─────────────────────────────────────────────────────────┐
│              Stellar / Soroban Blockchain                │
│                                                          │
│  ┌─────────────────┐  ┌─────────────────┐              │
│  │   Recurring     │  │    Metadata     │              │
│  │    Registry     │  │    Registry     │              │
│  │                 │  │                 │              │
│  │ subscriptions   │◄─┤ recurring_id    │              │
│  │ payments        │  │ categories      │              │
│  └─────────────────┘  └─────────────────┘              │
│                                                          │
│  ┌─────────────────┐  ┌─────────────────┐              │
│  │   Automation    │  │   Upgradeable   │              │
│  │     Rules       │  │   (Wrapper)     │              │
│  │                 │  │                 │              │
│  │ triggers        │  │ version mgmt    │              │
│  │ actions         │  │ admin control   │              │
│  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────┘
```
