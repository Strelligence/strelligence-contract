use soroban_sdk::{contracttype, Address, String};

/// Storage keys for the Metadata Registry.
///
/// - `Metadata(tx_hash)`        → persistent: the Metadata struct
/// - `WalletTxHashes(Address)`  → persistent: Vec<String> of tx hashes for a wallet
/// - `Admin`                    → instance: contract admin address
/// - `Version`                  → instance: contract version number
/// - `WasmHash`                 → instance: current WASM hash
#[contracttype]
pub enum DataKey {
    /// Full Metadata struct keyed by transaction hash
    Metadata(String),
    /// List of transaction hashes indexed for the given wallet
    WalletTxHashes(Address),
    /// Contract admin address for upgrade authority
    Admin,
    /// Contract version number for upgrade tracking
    Version,
    /// Current WASM hash for upgrade verification
    WasmHash,
}

/// Persistent storage TTL — approximately 1 year
pub const STORAGE_TTL: u32 = 3_110_400;

/// Initial contract version
pub const INITIAL_VERSION: u32 = 1;
