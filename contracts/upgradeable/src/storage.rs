use soroban_sdk::contracttype;

pub const INITIAL_VERSION: u32 = 1;

#[contracttype]
pub enum DataKey {
    Admin,
    Version,
    WasmHash,
}
