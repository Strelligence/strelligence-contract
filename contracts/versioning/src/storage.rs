use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Admin,
    VersionRecord(u64),
    NextRecordId,
    ContractVersion(Address),
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
