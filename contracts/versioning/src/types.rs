use soroban_sdk::{contracttype, Address, Bytes};

#[contracttype]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[contracttype]
pub struct VersionRecord {
    pub id: u64,
    pub contract_address: Address,
    pub version: SemanticVersion,
    pub wasm_hash: Bytes,
    pub created_at_ledger: u64,
    pub deprecated: bool,
}

#[contracttype]
pub struct CompatibilityCheck {
    pub from_version: SemanticVersion,
    pub to_version: SemanticVersion,
    pub compatible: bool,
    pub breaking_changes: bool,
}
