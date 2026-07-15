use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    /// No subscription found for the given ID
    SubscriptionNotFound = 1,
    /// Caller is not authorized to perform this action
    Unauthorized = 2,
    /// Subscription is already in cancelled state
    AlreadyCancelled = 3,
    /// Subscription is already in the requested state (e.g. pause a paused sub)
    AlreadyInState = 4,
    /// Amount must be greater than zero
    InvalidAmount = 5,
    /// Cannot confirm payment on a cancelled or expired subscription
    InactiveSubscription = 6,
    /// Contract has not been initialized
    NotInitialized = 7,
    /// Cannot upgrade to the same WASM hash
    SameWasmHash = 8,
    /// Caller is not the contract admin
    NotAdmin = 9,
}
