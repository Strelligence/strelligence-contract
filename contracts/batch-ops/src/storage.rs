use soroban_sdk::contracttype;

#[contracttype]
pub enum DataKey {
    Admin,
    BatchResult(u64),
    NextBatchId,
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
pub const MAX_BATCH_SIZE: u32 = 50;
