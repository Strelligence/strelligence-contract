# Contract Storage Analysis

Analysis of storage layout, costs, and optimization opportunities for each
Strelligence Soroban contract.

## Overview

Soroban uses three storage types:
- **Instance** — lives as long as the contract instance (global config)
- **Persistent** — lives until TTL expires (user data)
- **Temporary** — lives for a short period (caches)

All Strelligence contracts use:
- **Instance storage** for auto-increment counters and admin config
- **Persistent storage** for user data with 1-year TTL (3,110,400 ledgers)

---

## Recurring Registry

### Storage Keys

| Key | Type | Value |
|-----|------|-------|
| `Subscription(u64)` | Persistent | Full `Subscription` struct |
| `WalletSubscriptions(Address)` | Persistent | `Vec<u64>` of subscription IDs |
| `NextSubscriptionId` | Instance | `u64` auto-increment counter |

### Subscription Struct Size

| Field | Type | Approximate Size |
|-------|------|-----------------|
| `id` | u64 | 8 bytes |
| `owner` | Address | 32 bytes (StrKey) |
| `merchant` | String | 16 + N bytes |
| `merchant_address` | Option\<Address\> | 1 + 32 bytes |
| `frequency` | Frequency enum | 4 bytes |
| `subscription_type` | SubscriptionType enum | 4 bytes |
| `asset_code` | String | 16 + N bytes |
| `asset_issuer` | String | 16 + N bytes |
| `amount` | i128 | 16 bytes |
| `active` | bool | 1 byte |
| `status` | SubscriptionStatus enum | 4 bytes |
| `next_payment_ledger` | u64 | 8 bytes |
| `last_payment_ledger` | u64 | 8 bytes |
| `created_at_ledger` | u64 | 8 bytes |
| `auto_detected` | bool | 1 byte |
| `custom_label` | Option\<String\> | 1 + 16 + N bytes |
| **Total** | | **~160 + 2N bytes** |

### Storage Costs

| Operation | Storage Type | Cost (approx) |
|-----------|-------------|---------------|
| Create subscription | Persistent | ~200 bytes write |
| Update subscription | Persistent | ~200 bytes write |
| List wallet subscriptions | Persistent | ~8 bytes per ID |
| TTL extension | Persistent | 0.00001 XLM per entry |

### Index Structure

The `WalletSubscriptions` index is a `Vec<u64>` that grows linearly:
- 1 subscription = 8 bytes
- 10 subscriptions = 80 bytes
- 100 subscriptions = 800 bytes

**Optimization note:** For wallets with many subscriptions, consider paginated
reads rather than loading the entire index.

---

## Metadata Registry

### Storage Keys

| Key | Type | Value |
|-----|------|-------|
| `Metadata(tx_hash)` | Persistent | Full `Metadata` struct |
| `WalletTxHashes(Address)` | Persistent | `Vec<String>` of tx hashes |

### Metadata Struct Size

| Field | Type | Approximate Size |
|-------|------|-----------------|
| `tx_hash` | String | 16 + N bytes |
| `owner` | Address | 32 bytes |
| `category` | TransactionCategory enum | 4 bytes |
| `sentiment` | TransactionSentiment enum | 4 bytes |
| `tags` | Vec\<String\> | 16 + (16 + N) * count |
| `label` | Option\<String\> | 1 + 16 + N bytes |
| `notes` | Option\<String\> | 1 + 16 + N bytes |
| `counterparty_label` | Option\<String\> | 1 + 16 + N bytes |
| `is_recurring` | bool | 1 byte |
| `recurring_id` | Option\<u64\> | 1 + 8 bytes |
| `ai_confidence` | u32 | 4 bytes |
| `created_at_ledger` | u64 | 8 bytes |
| `updated_at_ledger` | u64 | 8 bytes |
| **Total** | | **~120 + 3N + 16*tags bytes** |

### Storage Costs

| Operation | Storage Type | Cost (approx) |
|-----------|-------------|---------------|
| Add metadata | Persistent | ~200 bytes write |
| Update metadata | Persistent | ~200 bytes write |
| List wallet metadata | Persistent | ~48 bytes per hash |
| Filter by category | Read-only | Linear scan |

### Index Structure

The `WalletTxHashes` index stores full tx hash strings:
- 1 metadata = ~48 bytes (tx hash string)
- 10 metadata = ~480 bytes
- 100 metadata = ~4,800 bytes

**Optimization note:** Tx hashes are 64-character hex strings. Consider using
a fixed-size byte array instead of Variable-length String to reduce overhead.

---

## Automation Rules

### Storage Keys

| Key | Type | Value |
|-----|------|-------|
| `Rule(u64)` | Persistent | Full `Rule` struct |
| `WalletRules(Address)` | Persistent | `Vec<u64>` of rule IDs |
| `NextRuleId` | Instance | `u64` auto-increment counter |

### Rule Struct Size

| Field | Type | Approximate Size |
|-------|------|-----------------|
| `id` | u64 | 8 bytes |
| `owner` | Address | 32 bytes |
| `rule_type` | RuleType enum | 4 bytes |
| `trigger` | RuleTrigger enum | 4 bytes |
| `status` | RuleStatus enum | 4 bytes |
| `label` | String | 16 + N bytes |
| `trigger_params` | String | 16 + N bytes (JSON) |
| `action_params` | String | 16 + N bytes (JSON) |
| `created_at_ledger` | u64 | 8 bytes |
| `last_executed_ledger` | u64 | 8 bytes |
| `execution_count` | u64 | 8 bytes |
| **Total** | | **~120 + 3N bytes** |

### Storage Costs

| Operation | Storage Type | Cost (approx) |
|-----------|-------------|---------------|
| Create rule | Persistent | ~160 bytes write |
| Update rule | Persistent | ~160 bytes write |
| Record execution | Persistent | ~160 bytes write |
| List wallet rules | Persistent | ~8 bytes per ID |

### Index Structure

Same as recurring registry — `Vec<u64>` of rule IDs.

---

## Upgradeable Contract

### Storage Keys

| Key | Type | Value |
|-----|------|-------|
| `Admin` | Instance | `Address` (32 bytes) |
| `Version` | Instance | `u32` (4 bytes) |
| `WasmHash` | Instance | `Bytes` (32 bytes) |

### Storage Costs

| Operation | Storage Type | Cost (approx) |
|-----------|-------------|---------------|
| Initialize | Instance | ~70 bytes write |
| Upgrade | Instance | ~40 bytes write |

---

## Optimization Opportunities

### 1. String Key Compression

**Current:** Tx hashes stored as full hex strings (64 chars = ~80 bytes)
**Optimized:** Use first 8 bytes of SHA-256 hash as key (8 bytes)
**Tradeoff:** Requires collision handling, slightly more complex reads

### 2. Vec Index Optimization

**Current:** Linear `Vec<u64>` or `Vec<String>` per wallet
**Optimized:** Paginated chunks (e.g., 100 IDs per entry)
**Tradeoff:** More complex queries, better scalability

### 3. Enum Size Reduction

**Current:** All enums use 4 bytes (default Soroban encoding)
**Optimized:** Custom encoding with `u8` discriminants (1 byte)
**Tradeoff:** Requires custom serialization, less readable

### 4. Optional Field Consolidation

**Current:** Each `Option<T>` adds 1-byte discriminant + 16-byte tag
**Optimized:** Use sentinel values or separate "flags" field
**Tradeoff:** Less idiomatic, slightly smaller structs

### 5. JSON String Optimization

**Current:** Trigger/action params stored as JSON strings
**Optimized:** Use structured `Map<String, Val>` types
**Tradeoff:** More complex parsing, better type safety

---

## Recommendations

1. **Short-term:** No changes needed. Current storage costs are minimal for
   typical usage (< 100 records per wallet).

2. **Medium-term:** If scaling to > 1000 records per wallet, implement
   paginated Vec indexes (recommendation #2).

3. **Long-term:** For high-throughput scenarios, consider:
   - Off-chain indexing with on-chain anchors
   - Batch operations to reduce per-transaction costs
   - Storage quotas per wallet to prevent bloat

## Cost Summary

| Contract | Per-Record Cost | 100 Records | 1000 Records |
|----------|----------------|-------------|--------------|
| Recurring Registry | ~200 bytes | ~20 KB | ~200 KB |
| Metadata Registry | ~250 bytes | ~25 KB | ~250 KB |
| Automation Rules | ~160 bytes | ~16 KB | ~160 KB |

At current Stellar storage costs (~0.00001 XLM per byte per year),
1000 records per contract costs approximately 0.005 XLM/year.
