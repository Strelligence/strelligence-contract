# Security Audit Checklist

This document covers common Soroban smart contract vulnerabilities and how the
Strelligence contracts address them.

## Authentication & Authorization

| Check | Status | Details |
|-------|--------|---------|
| All write functions require auth | ✅ | `caller.require_auth()` in every mutating function |
| Owner-only operations verified | ✅ | `owner.require_auth()` for update/cancel/pause/delete |
| No privilege escalation possible | ✅ | Dual auth: caller + owner checked separately |

**Contract references:**
- `recurring-registry/src/contract.rs` — lines 38, 122, 170, 202, 235, 268
- `metadata-registry/src/contract.rs` — lines 31, 95
- `automation-rules/src/contract.rs` — lines 29, 89, 131, 158, 185, 209

## Storage Safety

| Check | Status | Details |
|-------|--------|---------|
| Persistent storage used correctly | ✅ | `.persistent().set()` for all mutable data |
| TTL set on all persistent entries | ✅ | `extend_ttl()` called after every write with `STORAGE_TTL` (3,110,400 ledgers ≈ 1 year) |
| No storage bloat vectors | ✅ | Index vectors (`Vec<u64>`, `Vec<String>`) grow linearly with user data only |
| Instance storage for globals | ✅ | Auto-increment counters use `.instance()` storage |

**Contract references:**
- `recurring-registry/src/storage.rs` — `STORAGE_TTL` constant
- `metadata-registry/src/storage.rs` — `STORAGE_TTL` constant
- `automation-rules/src/storage.rs` — `STORAGE_TTL` constant

## Input Validation

| Check | Status | Details |
|-------|--------|---------|
| Amount > 0 checks | ✅ | `if amount <= 0 { return Err(InvalidAmount) }` in recurring-registry |
| Confidence score 0–100 | ✅ | `if ai_confidence > 100 { return Err(InvalidConfidence) }` in metadata-registry |
| Duplicate prevention | ✅ | `has()` check before `add_metadata` prevents duplicate tx_hash records |
| State-based guards | ✅ | Cannot pause a deleted rule, cannot cancel an already-cancelled subscription |

**Contract references:**
- `recurring-registry/src/contract.rs:40` — amount validation
- `metadata-registry/src/contract.rs:37` — confidence validation
- `metadata-registry/src/contract.rs:41` — duplicate check

## State Machine

| Check | Status | Details |
|-------|--------|---------|
| Valid state transitions only | ✅ | Subscription: Active → Paused → Active, Active → Cancelled |
| No double-action | ✅ | Already-cancelled / already-deleted checks prevent repeat operations |
| Idempotent reads | ✅ | `get_*` and `list_*` functions are pure reads with no side effects |

**State diagrams:**

```
Subscription:  Active ⇄ Paused → Cancelled
Rule:          Active ⇄ Paused → Deleted
```

## Error Handling

| Check | Status | Details |
|-------|--------|---------|
| All error paths return proper errors | ✅ | Every fallible path returns `Result<T, ContractError>` |
| No unwrap() in production code | ✅ | Only `.unwrap_or()` for defaults (safe) |
| Descriptive error variants | ✅ | `SubscriptionNotFound`, `InvalidAmount`, `AlreadyCancelled`, etc. |
| Option returns for reads | ✅ | `get_subscription`, `get_metadata`, `get_rule` return `Option<T>` |

## Event Emission

| Check | Status | Details |
|-------|--------|---------|
| All mutations emit events | ✅ | Created, updated, cancelled, paused, deleted, executed |
| Events include owner address | ✅ | Every event includes the owner in topics |
| Events include entity ID | ✅ | Subscription ID, tx hash, or rule ID in event data |

## Known Limitations

1. **No on-chain enforcement of recurring payments** — The recurring-registry is
   a metadata registry only. Payment execution happens off-chain via the backend.
   The contract tracks state but does not enforce payment timing.

2. **Backend trust model** — The `record_execution` and `confirm_payment`
   functions are called by the backend. The contract trusts the caller for
   timestamp/ledger values. A compromised backend could report incorrect
   execution data.

3. **No built-in upgrade mechanism** — Contract logic is immutable after
   deployment. Upgrades require deploying a new contract and migrating state.

4. **Single-owner model** — Each subscription, metadata record, and rule has
   exactly one owner. There is no multi-sig or shared ownership support.

5. **String-based indexing** — Transaction hashes and wallet addresses are stored
   as Soroban `String` types. Large numbers of records per wallet will increase
   read costs linearly.

6. **No rate limiting** — There is no on-chain rate limiting for metadata writes
   or rule creation. A malicious actor could fill storage for a single wallet.
