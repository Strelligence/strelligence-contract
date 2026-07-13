# Metadata Registry

Stores standardized financial metadata attached to Stellar transactions.

## Overview

The Metadata Registry contract attaches AI-classified categories, sentiment,
tags, and notes to individual transaction hashes. It provides wallet-level
indexing and category-based filtering.

## Functions

### Write Functions

#### `add_metadata`

Attach metadata to a transaction hash.

```rust
pub fn add_metadata(
    env: Env,
    caller: Address,
    tx_hash: String,
    category: TransactionCategory,
    sentiment: TransactionSentiment,
    tags: Vec<String>,
    label: Option<String>,
    notes: Option<String>,
    counterparty_label: Option<String>,
    is_recurring: bool,
    recurring_id: Option<u64>,
    ai_confidence: u32,
) -> Result<(), ContractError>
```

**Errors:**
- `AlreadyExists` — metadata already exists for this tx_hash
- `InvalidConfidence` — confidence > 100

#### `update_metadata`

Update mutable fields of existing metadata. Owner only.

```rust
pub fn update_metadata(
    env: Env,
    caller: Address,
    tx_hash: String,
    category: Option<TransactionCategory>,
    sentiment: Option<TransactionSentiment>,
    tags: Option<Vec<String>>,
    label: Option<String>,
    notes: Option<String>,
    counterparty_label: Option<String>,
) -> Result<(), ContractError>
```

**Errors:**
- `MetadataNotFound`
- `Unauthorized`

### Read Functions

#### `get_metadata`

Fetch metadata for a transaction hash.

```rust
pub fn get_metadata(env: Env, tx_hash: String) -> Option<Metadata>
```

#### `get_wallet_metadata`

List all transaction hashes with metadata for a wallet.

```rust
pub fn get_wallet_metadata(env: Env, owner: Address) -> Vec<String>
```

#### `get_metadata_by_category`

Filter metadata by category for a wallet.

```rust
pub fn get_metadata_by_category(
    env: Env,
    owner: Address,
    category: TransactionCategory,
) -> Vec<Metadata>
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | MetadataNotFound | No metadata for this tx_hash |
| 2 | Unauthorized | Caller is not the owner |
| 3 | AlreadyExists | Metadata already exists for tx_hash |
| 4 | InvalidConfidence | Confidence must be 0–100 |

## Types

### TransactionCategory
`Income`, `Expense`, `Transfer`, `Subscription`, `Savings`, `Payroll`, `Investment`, `Fee`, `Swap`, `Unknown`

### TransactionSentiment
`Positive`, `Neutral`, `Negative`

### Tag
`String` — short label applied to a transaction

## Storage Layout

| Key | Storage Type | Value |
|-----|-------------|-------|
| `Metadata(tx_hash)` | Persistent | Full `Metadata` struct |
| `WalletTxHashes(owner)` | Persistent | `Vec<String>` of tx hashes |

## Usage Example

```rust
use metadata_registry::{MetadataRegistryContract, TransactionCategory, TransactionSentiment};

let client = MetadataRegistryContractClient::new(&env, &contract_id);

client.add_metadata(
    &owner,
    &String::from_str(&env, "tx_abc123"),
    &TransactionCategory::Subscription,
    &TransactionSentiment::Negative,
    &soroban_sdk::vec![&env, String::from_str(&env, "streaming")],
    &Some(String::from_str(&env, "Netflix Payment")),
    &Some(String::from_str(&env, "Monthly subscription")),
    &Some(String::from_str(&env, "Netflix")),
    &true,
    &Some(42),
    &95,
);

let meta = client.get_metadata(&String::from_str(&env, "tx_abc123")).unwrap();
assert_eq!(meta.category, TransactionCategory::Subscription);
```
