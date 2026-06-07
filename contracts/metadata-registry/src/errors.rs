use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    /// No metadata found for the given transaction hash
    MetadataNotFound = 1,
    /// Caller is not the owner of the metadata record
    Unauthorized = 2,
    /// Metadata already exists for this transaction hash
    AlreadyExists = 3,
    /// Confidence score must be between 0 and 100
    InvalidConfidence = 4,
}