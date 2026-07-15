use soroban_sdk::{contracttype, Address, Bytes};

#[contracttype]
#[derive(Clone)]
pub struct BatchOperation {
    pub id: u64,
    pub operation_type: BatchOpType,
    pub target: Address,
    pub payload: Bytes,
    pub status: BatchOpStatus,
}

#[contracttype]
#[derive(Clone)]
pub enum BatchOpType {
    Create,
    Update,
    Delete,
}

#[contracttype]
#[derive(Clone)]
pub enum BatchOpStatus {
    Pending,
    Success,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BatchResult {
    pub batch_id: u64,
    pub total_operations: u32,
    pub successful: u32,
    pub failed: u32,
    pub status: BatchStatus,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum BatchStatus {
    Completed,
    PartialFailure,
    AllFailed,
}
