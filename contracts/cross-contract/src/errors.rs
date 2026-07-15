use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NotAdmin = 4,
    ContractCallFailed = 5,
    InvalidContract = 6,
    AlreadyProcessed = 7,
    SameWasmHash = 8,
}
