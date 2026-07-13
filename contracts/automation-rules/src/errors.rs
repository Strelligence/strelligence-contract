use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    RuleNotFound = 1,
    Unauthorized = 2,
    AlreadyDeleted = 3,
    InvalidParams = 4,
}
