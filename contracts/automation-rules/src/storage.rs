use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Rule(u64),
    WalletRules(Address),
    NextRuleId,
    Admin,
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;

pub const INITIAL_VERSION: u32 = 1;
