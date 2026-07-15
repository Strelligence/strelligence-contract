use soroban_sdk::{contracttype, Address, String};

#[contracttype]
pub struct CrossContractCall {
    pub id: u64,
    pub caller: Address,
    pub target_contract: Address,
    pub function_name: String,
    pub status: CallStatus,
    pub created_at_ledger: u64,
    pub completed_at_ledger: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum CallStatus {
    Pending,
    Success,
    Failed,
}

#[contracttype]
pub struct ContractRegistry {
    pub recurring_registry: Address,
    pub metadata_registry: Address,
    pub automation_rules: Address,
}
