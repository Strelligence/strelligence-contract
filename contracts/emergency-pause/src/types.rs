use soroban_sdk::{contracttype, Address};

#[contracttype]
pub struct PauseState {
    pub paused: bool,
    pub paused_by: Address,
    pub paused_at_ledger: u64,
    pub unpause_at_ledger: Option<u64>,
    pub reason: soroban_sdk::String,
}

#[contracttype]
pub struct PauseHistory {
    pub pause_count: u64,
    pub last_paused_at: u64,
    pub last_unpaused_at: u64,
}
