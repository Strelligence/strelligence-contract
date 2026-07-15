use soroban_sdk::{contracttype, Address, Bytes, String};

#[contracttype]
pub struct Snapshot {
    pub id: u64,
    pub creator: Address,
    pub contract_address: Address,
    pub label: String,
    pub state_hash: Bytes,
    pub created_at_ledger: u64,
    pub expired: bool,
}

#[contracttype]
#[derive(Debug)]
pub struct SnapshotDiff {
    pub snapshot_a_id: u64,
    pub snapshot_b_id: u64,
    pub identical: bool,
    pub added_keys: u32,
    pub removed_keys: u32,
    pub modified_keys: u32,
}
