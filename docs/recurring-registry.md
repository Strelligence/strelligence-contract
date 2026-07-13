# Recurring Registry

Tracks recurring payment relationships on Stellar/Soroban.

## Overview

The Recurring Registry contract stores subscription records for recurring payments.
Each record tracks the merchant, frequency, amount, asset, and lifecycle state.

## Functions

### Write Functions

#### `create_subscription`

Register a new recurring payment.

```rust
pub fn create_subscription(
    env: Env,
    caller: Address,
    owner: Address,
    merchant: String,
    merchant_address: Option<Address>,
    frequency: Frequency,
    subscription_type: SubscriptionType,
    asset_code: String,
    asset_issuer: String,
    amount: i128,
    next_payment_ledger: u64,
    auto_detected: bool,
    custom_label: Option<String>,
) -> Result<u64, ContractError>
```

**Returns:** Subscription ID (u64)

**Errors:**
- `InvalidAmount` — amount <= 0

#### `update_subscription`

Update mutable fields of an existing subscription. Owner only.

```rust
pub fn update_subscription(
    env: Env,
    caller: Address,
    id: u64,
    merchant: Option<String>,
    frequency: Option<Frequency>,
    amount: Option<i128>,
    next_payment_ledger: Option<u64>,
    custom_label: Option<String>,
) -> Result<(), ContractError>
```

**Errors:**
- `SubscriptionNotFound` — ID does not exist
- `InvalidAmount` — new amount <= 0

#### `cancel_subscription`

Permanently cancel a subscription. Owner only.

```rust
pub fn cancel_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `SubscriptionNotFound`
- `AlreadyCancelled`

#### `pause_subscription`

Temporarily pause a subscription. Owner only.

```rust
pub fn pause_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `SubscriptionNotFound`
- `AlreadyInState` — already paused
- `AlreadyCancelled`

#### `resume_subscription`

Resume a paused subscription. Owner only.

```rust
pub fn resume_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `SubscriptionNotFound`
- `AlreadyCancelled`

#### `confirm_payment`

Backend calls this after confirming a payment occurred.

```rust
pub fn confirm_payment(
    env: Env,
    caller: Address,
    id: u64,
    payment_ledger: u64,
    next_payment_ledger: u64,
) -> Result<(), ContractError>
```

**Errors:**
- `SubscriptionNotFound`
- `InactiveSubscription` — cancelled or expired

### Read Functions

#### `get_subscription`

Fetch a single subscription by ID.

```rust
pub fn get_subscription(env: Env, id: u64) -> Option<Subscription>
```

#### `list_wallet_subscriptions`

List all subscription IDs for a wallet.

```rust
pub fn list_wallet_subscriptions(env: Env, owner: Address) -> Vec<u64>
```

#### `list_active_subscriptions`

List only active subscriptions for a wallet.

```rust
pub fn list_active_subscriptions(env: Env, owner: Address) -> Vec<Subscription>
```

#### `total_subscriptions`

Total subscriptions ever created.

```rust
pub fn total_subscriptions(env: Env) -> u64
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | SubscriptionNotFound | No subscription with this ID |
| 2 | Unauthorized | Caller is not the owner |
| 3 | AlreadyCancelled | Subscription already cancelled |
| 4 | AlreadyInState | Subscription already in requested state |
| 5 | InvalidAmount | Amount must be > 0 |
| 6 | InactiveSubscription | Cannot operate on cancelled/expired subscription |

## Types

### Frequency
`Daily`, `Weekly`, `BiWeekly`, `Monthly`, `Quarterly`, `Annually`, `Custom`

### SubscriptionType
`Subscription`, `Payroll`, `Income`, `Savings`, `Bill`, `Investment`, `Transfer`, `Other`

### SubscriptionStatus
`Active`, `Paused`, `Cancelled`, `Expired`

## Storage Layout

| Key | Storage Type | Value |
|-----|-------------|-------|
| `Subscription(id)` | Persistent | Full `Subscription` struct |
| `WalletSubscriptions(owner)` | Persistent | `Vec<u64>` of subscription IDs |
| `NextSubscriptionId` | Instance | Auto-increment counter |

## Usage Example

```rust
use recurring_registry::{RecurringRegistryContract, Frequency, SubscriptionType};

let client = RecurringRegistryContractClient::new(&env, &contract_id);

let id = client.create_subscription(
    &owner,
    &owner,
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

let sub = client.get_subscription(&id).unwrap();
assert_eq!(sub.status, SubscriptionStatus::Active);
```
