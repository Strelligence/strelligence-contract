# Contract Events Reference

All Strelligence contracts emit events on state-changing operations. This
document describes every event's topic signature, data format, and usage for
building indexers and webhook systems.

---

## Recurring Registry Events

Source: `contracts/recurring-registry/src/events.rs`

### `sub_crtd` — Subscription Created

Emitted when a new subscription is registered.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"sub_crtd"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Subscription ID |

### `sub_upd` — Subscription Updated

Emitted when a subscription's mutable fields are changed.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"sub_upd"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Subscription ID |

### `sub_can` — Subscription Cancelled

Emmitted when a subscription is permanently cancelled.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"sub_can"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Subscription ID |

### `sub_psd` — Subscription Paused

Emitted when a subscription is temporarily paused.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"sub_psd"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Subscription ID |

### `pay_conf` — Payment Confirmed

Emitted when the backend confirms a payment occurred.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"pay_conf"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Subscription ID |

---

## Metadata Registry Events

Source: `contracts/metadata-registry/src/events.rs`

### `meta_add` — Metadata Added

Emitted when metadata is first attached to a transaction hash.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"meta_add"` |
| Topic 1 | Address | Owner wallet address |
| Data | String | Transaction hash |

### `meta_upd` — Metadata Updated

Emitted when existing metadata is modified by the owner.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"meta_upd"` |
| Topic 1 | Address | Owner wallet address |
| Data | String | Transaction hash |

---

## Automation Rules Events

Source: `contracts/automation-rules/src/events.rs`

### `rule_cr` — Rule Created

Emitted when a new automation rule is registered.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"rule_cr"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Rule ID |

### `rule_up` — Rule Updated

Emitted when a rule's mutable fields are changed.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"rule_up"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Rule ID |

### `rule_ps` — Rule Paused

Emitted when a rule is paused.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"rule_ps"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Rule ID |

### `rule_dl` — Rule Deleted

Emitted when a rule is soft-deleted.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"rule_dl"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Rule ID |

### `rule_ex` — Rule Executed

Emitted when the backend records a rule execution.

| Field | Type | Description |
|-------|------|-------------|
| Topic 0 | Symbol | `"rule_ex"` |
| Topic 1 | Address | Owner wallet address |
| Data | u64 | Rule ID |

---

## Parsing Events

### JavaScript (Stellar SDK)

```javascript
import { SorobanRpc } from "@stellar/stellar-sdk";

const server = new SorobanRpc.Server("https://soroban-testnet.stellar.org");

const events = await server.getEvents({
  contractId: CONTRACT_ID,
  startLedger: 100000,
  limit: 10,
});

for (const event of events.events) {
  const [topic0, owner] = event.topics;
  const topic0Str = topic0.value().toString();

  switch (topic0Str) {
    case "sub_crtd": {
      const subId = event.data.value();
      console.log(`Subscription ${subId} created by ${owner}`);
      break;
    }
    case "meta_add": {
      const txHash = event.data.value().toString();
      console.log(`Metadata added for tx ${txHash} by ${owner}`);
      break;
    }
    case "rule_cr": {
      const ruleId = event.data.value();
      console.log(`Rule ${ruleId} created by ${owner}`);
      break;
    }
  }
}
```

### Rust (soroban-sdk)

```rust
use soroban_sdk::{symbol_short, Env, Address};

fn filter_events(env: &Env, contract_id: &Address) {
    let events = env.events().all();
    for event in events {
        let topics = event.topics();
        let topic0 = topics.get_unchecked(0);

        if topic0 == symbol_short!("sub_crtd").to_val() {
            let owner = topics.get_unchecked(1);
            let sub_id: u64 = event.data().try_into().unwrap();
            log!(env, "Subscription {} created by {}", sub_id, owner);
        }
    }
}
```

### Stellar Indexer (Rust)

```rust
use stellar_xdr::ContractEvent;

fn handle_event(event: &ContractEvent) {
    let topics = &event.topics;
    let symbol = &topics[0]; // Event type symbol
    let owner = &topics[1];  // Owner address
    let data = &event.data;  // Entity ID or tx hash

    match symbol.to_string().as_str() {
        "sub_crtd" | "sub_upd" | "sub_can" | "sub_psd" | "pay_conf" => {
            // Recurring registry event
            let entity_id: u64 = data.clone().try_into().unwrap();
            println!("Recurring event: {} for entity {}", symbol, entity_id);
        }
        "meta_add" | "meta_upd" => {
            // Metadata registry event
            let tx_hash: String = data.clone().try_into().unwrap();
            println!("Metadata event: {} for tx {}", symbol, tx_hash);
        }
        "rule_cr" | "rule_up" | "rule_ps" | "rule_dl" | "rule_ex" => {
            // Automation rules event
            let rule_id: u64 = data.clone().try_into().unwrap();
            println!("Rule event: {} for rule {}", symbol, rule_id);
        }
        _ => {}
    }
}
```
