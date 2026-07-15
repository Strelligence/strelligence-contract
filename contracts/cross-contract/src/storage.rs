use soroban_sdk::contracttype;

#[contracttype]
pub enum DataKey {
    Admin,
    ContractRegistry,
    CrossCall(u64),
    NextCallId,
    ProcessedCall(u64),
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
