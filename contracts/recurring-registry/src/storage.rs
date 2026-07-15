use soroban_sdk::{contracttype, Address};

/// All storage keys used by the Recurring Registry contract.
///
/// Design:
/// - `Subscription(u64)`         → persistent: the full Subscription struct
/// - `WalletSubscriptions(Address)` → persistent: Vec<u64> of subscription IDs for a wallet
/// - `NextSubscriptionId`        → instance: global auto-increment counter
/// - `Admin`                     → instance: contract admin address
/// - `Version`                   → instance: contract version number
/// - `WasmHash`                  → instance: current WASM hash
#[contracttype]
pub enum DataKey {
    /// Stores a Subscription struct, keyed by its numeric ID
    Subscription(u64),
    /// Stores Vec<u64> of subscription IDs owned by the given wallet
    WalletSubscriptions(Address),
    /// Stores the next available subscription ID (u64)
    NextSubscriptionId,
    /// Contract admin address for upgrade authority
    Admin,
    /// Contract version number for upgrade tracking
    Version,
    /// Current WASM hash for upgrade verification
    WasmHash,
}

/// Storage TTL in ledgers — approximately 1 year on Stellar mainnet
/// (5 seconds per ledger × 365 days × 24 hrs × 720 ledgers/hr)
pub const STORAGE_TTL: u32 = 3_110_400;

/// Initial contract version
pub const INITIAL_VERSION: u32 = 1;
