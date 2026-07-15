use soroban_sdk::contracttype;

#[contracttype]
pub enum DataKey {
    Admin,
    PauseState,
    PauseHistory,
    Version,
    WasmHash,
}

pub const STORAGE_TTL: u32 = 3_110_400;
pub const INITIAL_VERSION: u32 = 1;
