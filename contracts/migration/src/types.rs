use soroban_sdk::{contracttype, Address, Bytes};

#[contracttype]
pub struct MigrationPlan {
    pub id: u64,
    pub contract_address: Address,
    pub from_version: u32,
    pub to_version: u32,
    pub from_wasm_hash: Bytes,
    pub to_wasm_hash: Bytes,
    pub status: MigrationStatus,
    pub created_at_ledger: u64,
    pub completed_at_ledger: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationStatus {
    Pending,
    InProgress,
    Completed,
    RolledBack,
    Failed,
}

#[contracttype]
#[derive(Debug)]
pub struct VerificationResult {
    pub plan_id: u64,
    pub storage_valid: bool,
    pub state_valid: bool,
    pub events_valid: bool,
}
