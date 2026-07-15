use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NotAdmin = 4,
    SameWasmHash = 5,
    AlreadyPaused = 6,
    NotPaused = 7,
    ContractPaused = 8,
}
