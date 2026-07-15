use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Admin,
    EventRecord(u64),
    NextEventId,
    TopicIndex(soroban_sdk::Symbol, u64),
    LedgerIndex(u64, u64),
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
pub const MAX_PAGE_SIZE: u32 = 100;
