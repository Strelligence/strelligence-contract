use soroban_sdk::{contracttype, Address, Symbol};

#[contracttype]
pub struct EventRecord {
    pub id: u64,
    pub contract_address: Address,
    pub topic: Symbol,
    pub data: soroban_sdk::Bytes,
    pub ledger: u64,
    pub timestamp: u64,
}

#[contracttype]
pub struct EventFilter {
    pub topic: Option<Symbol>,
    pub from_ledger: Option<u64>,
    pub to_ledger: Option<u64>,
    pub contract_address: Option<Address>,
}

#[contracttype]
pub struct PaginatedEvents {
    pub events: soroban_sdk::Vec<EventRecord>,
    pub next_cursor: Option<u64>,
    pub total_count: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct EventAggregation {
    pub topic: Symbol,
    pub count: u64,
    pub first_ledger: u64,
    pub last_ledger: u64,
}
