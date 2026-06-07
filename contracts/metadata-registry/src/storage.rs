use soroban_sdk::{contracttype, Address, String};

/// Storage keys for the Metadata Registry.
///
/// - `Metadata(tx_hash)`        → persistent: the Metadata struct
/// - `WalletTxHashes(Address)`  → persistent: Vec<String> of tx hashes for a wallet
#[contracttype]
pub enum DataKey {
    /// Full Metadata struct keyed by transaction hash
    Metadata(String),
    /// List of transaction hashes indexed for the given wallet
    WalletTxHashes(Address),
}

/// Persistent storage TTL — approximately 1 year
pub const STORAGE_TTL: u32 = 3_110_400;