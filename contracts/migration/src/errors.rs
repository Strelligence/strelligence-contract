use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NotAdmin = 4,
    PlanNotFound = 5,
    InvalidMigrationStatus = 6,
    SameWasmHash = 7,
    VerificationFailed = 8,
}
