use soroban_sdk::{contracttype, Address};

/// All storage keys used by the Recurring Registry contract.
///
/// # Storage Layout Strategy
///
/// This contract uses a tiered storage strategy to optimize gas costs:
///
/// ## Instance Storage (lowest cost, shared across all calls)
/// - `NextSubscriptionId` → global auto-increment counter
/// - `Admin` → contract admin address for upgrade authority
/// - `Version` → contract version number for upgrade tracking
/// - `WasmHash` → current WASM hash for upgrade verification
///
/// ## Persistent Storage (higher cost, but survives across transactions)
/// - `Subscription(u64)` → the full Subscription struct
/// - `WalletSubscriptions(Address)` → Vec<u64> of subscription IDs for a wallet
///
/// # Gas Cost Optimization Notes
///
/// 1. **Instance storage is cheapest** — use for global config (admin, version, counters)
/// 2. **Persistent storage is expensive** — use only for data that must survive across transactions
/// 3. **TTL management** — extend TTL only on write operations to minimize storage costs
/// 4. **Index design** — wallet indexes use Vec<u64> to avoid nested storage reads
/// 5. **Struct packing** — Subscription struct uses fixed-size fields where possible
///
/// # Storage Cost Estimates (Stellar mainnet)
///
/// - Instance write: ~0.00001 XLM per 4KB
/// - Persistent write: ~0.0001 XLM per 4KB
/// - TTL extension: ~0.00001 XLM per ledger
#[contracttype]
pub enum DataKey {
    /// Stores a Subscription struct, keyed by its numeric ID
    /// Cost: ~0.001 XLM for initial write + TTL extension
    Subscription(u64),
    /// Stores Vec<u64> of subscription IDs owned by the given wallet
    /// Cost: ~0.0001 XLM per ID added (amortized)
    WalletSubscriptions(Address),
    /// Stores the next available subscription ID (u64)
    /// Cost: ~0.00001 XLM (instance storage)
    NextSubscriptionId,
    /// Contract admin address for upgrade authority
    /// Cost: ~0.00001 XLM (instance storage)
    Admin,
    /// Contract version number for upgrade tracking
    /// Cost: ~0.00001 XLM (instance storage)
    Version,
    /// Current WASM hash for upgrade verification
    /// Cost: ~0.00001 XLM (instance storage)
    WasmHash,
}

/// Storage TTL in ledgers — approximately 1 year on Stellar mainnet
/// (5 seconds per ledger × 365 days × 24 hrs × 720 ledgers/hr)
pub const STORAGE_TTL: u32 = 3_110_400;

/// Initial contract version
pub const INITIAL_VERSION: u32 = 1;

/// Maximum number of subscriptions per wallet (gas safety limit)
pub const MAX_SUBSCRIPTIONS_PER_WALLET: u32 = 100;

/// Storage cost multiplier for persistent vs instance storage
/// Persistent storage costs ~10x more than instance storage
pub const PERSISTENT_STORAGE_COST_MULTIPLIER: u32 = 10;

/// Estimated base cost in stroops for a single persistent storage write
pub const PERSISTENT_WRITE_BASE_COST: u64 = 100_000;

/// Estimated base cost in stroops for a single instance storage write
pub const INSTANCE_WRITE_BASE_COST: u64 = 10_000;

/// Maximum recommended subscription amount (prevents overflow in calculations)
pub const MAX_SUBSCRIPTION_AMOUNT: i128 = i128::MAX / 2;

/// Default page size for paginated queries
pub const DEFAULT_PAGE_SIZE: u32 = 50;

/// Maximum page size for paginated queries
pub const MAX_PAGE_SIZE: u32 = 200;
