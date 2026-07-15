use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NotAdmin = 4,
    RoleNotFound = 5,
    RoleAlreadyAssigned = 6,
    InsufficientPermissions = 7,
    SameWasmHash = 8,
    RoleExpired = 9,
}
