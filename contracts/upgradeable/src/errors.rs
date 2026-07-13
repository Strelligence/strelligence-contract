use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    Unauthorized = 2,
    SameWasmHash = 3,
}
