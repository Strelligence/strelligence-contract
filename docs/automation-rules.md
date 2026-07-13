# Automation Rules

Supports programmable financial routing and treasury logic.

## Overview

The Automation Rules contract registers rules with triggers and actions that
the backend executes. The contract is the source-of-truth registry — it does
not execute rules itself.

## Functions

### Write Functions

#### `create_rule`

Create a new automation rule.

```rust
pub fn create_rule(
    env: Env,
    caller: Address,
    rule_type: RuleType,
    trigger: RuleTrigger,
    label: String,
    trigger_params: String,
    action_params: String,
) -> Result<u64, ContractError>
```

**Returns:** Rule ID (u64)

#### `update_rule`

Update mutable fields of an existing rule. Owner only.

```rust
pub fn update_rule(
    env: Env,
    caller: Address,
    id: u64,
    label: Option<String>,
    trigger_params: Option<String>,
    action_params: Option<String>,
) -> Result<(), ContractError>
```

**Errors:**
- `RuleNotFound`

#### `pause_rule`

Pause an active rule. Owner only.

```rust
pub fn pause_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `RuleNotFound`
- `AlreadyDeleted`

#### `resume_rule`

Resume a paused rule. Owner only.

```rust
pub fn resume_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `RuleNotFound`
- `AlreadyDeleted`

#### `delete_rule`

Soft-delete a rule. Owner only.

```rust
pub fn delete_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `RuleNotFound`
- `AlreadyDeleted`

#### `record_execution`

Backend calls this after executing a rule.

```rust
pub fn record_execution(env: Env, caller: Address, id: u64) -> Result<(), ContractError>
```

**Errors:**
- `RuleNotFound`

### Read Functions

#### `get_rule`

Fetch a rule by ID.

```rust
pub fn get_rule(env: Env, id: u64) -> Option<Rule>
```

#### `list_wallet_rules`

List all rule IDs for a wallet.

```rust
pub fn list_wallet_rules(env: Env, owner: Address) -> Vec<u64>
```

#### `list_active_rules`

List only active rules for a wallet.

```rust
pub fn list_active_rules(env: Env, owner: Address) -> Vec<Rule>
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | RuleNotFound | No rule with this ID |
| 2 | Unauthorized | Caller is not the owner |
| 3 | AlreadyDeleted | Rule already deleted |
| 4 | InvalidParams | Invalid parameters |

## Types

### RuleType
`AutoSave`, `AutoSweep`, `Payroll`, `Budget`, `Alert`

### RuleTrigger
`OnIncomingPayment`, `OnOutgoingPayment`, `OnSchedule`, `OnBalanceAbove`, `OnBalanceBelow`, `OnCategorySpend`

### RuleStatus
`Active`, `Paused`, `Deleted`

## Storage Layout

| Key | Storage Type | Value |
|-----|-------------|-------|
| `Rule(id)` | Persistent | Full `Rule` struct |
| `WalletRules(owner)` | Persistent | `Vec<u64>` of rule IDs |
| `NextRuleId` | Instance | Auto-increment counter |

## Usage Example

```rust
use automation_rules::{AutomationRulesContract, RuleType, RuleTrigger};

let client = AutomationRulesContractClient::new(&env, &contract_id);

let id = client.create_rule(
    &owner,
    &RuleType::AutoSave,
    &RuleTrigger::OnIncomingPayment,
    &String::from_str(&env, "Auto-save 10%"),
    &String::from_str(&env, r#"{"percentage":10}"#),
    &String::from_str(&env, r#"{"dest":"G..."}"#),
);

let rule = client.get_rule(&id).unwrap();
assert_eq!(rule.status, RuleStatus::Active);
```
