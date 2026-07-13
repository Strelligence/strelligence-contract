use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Rule(u64),
    WalletRules(Address),
    NextRuleId,
}

pub const STORAGE_TTL: u32 = 3_110_400;
