use soroban_sdk::contracttype;

#[contracttype]
pub enum DataKey {
    Admin,
    GasProfile(u64),
    NextProfileId,
    OptimizationSettings,
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
pub const DEFAULT_BATCH_SIZE: u32 = 100;
