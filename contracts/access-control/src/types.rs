use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Role {
    Admin,
    Operator,
    Viewer,
}

#[contracttype]
pub struct RoleAssignment {
    pub address: Address,
    pub role: Role,
    pub granted_at_ledger: u64,
    pub expires_at_ledger: Option<u64>,
}

#[contracttype]
pub struct Permission {
    pub resource: u32,
    pub action: u32,
}
