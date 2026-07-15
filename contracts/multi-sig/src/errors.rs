use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NotAdmin = 4,
    SignerAlreadyExists = 5,
    SignerNotFound = 6,
    ThresholdExceedsSigners = 7,
    InvalidThreshold = 8,
    ProposalNotFound = 9,
    AlreadySigned = 10,
    AlreadyExecuted = 11,
    NotReadyToExecute = 12,
    InvalidSignerCount = 13,
    CannotRemoveSelf = 14,
    SignerInactive = 15,
    SameWasmHash = 16,
}
