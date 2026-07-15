use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct Signer {
    pub address: Address,
    pub added_at_ledger: u64,
    pub active: bool,
}

#[contracttype]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub target_contract: Address,
    pub function_name: soroban_sdk::String,
    pub payload: soroban_sdk::Bytes,
    pub created_at_ledger: u64,
    pub executed: bool,
    pub cancelled: bool,
}

#[contracttype]
pub enum ProposalStatus {
    Pending,
    Executed,
    Cancelled,
    Expired,
}
