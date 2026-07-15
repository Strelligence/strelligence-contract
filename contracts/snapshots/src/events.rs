use soroban_sdk::{symbol_short, Address, Env};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

pub fn snapshot_created(env: &Env, creator: &Address, snapshot_id: u64) {
    let topics = (symbol_short!("v1_snp_cr"), creator.clone());
    env.events().publish(topics, snapshot_id);
}

pub fn snapshot_deleted(env: &Env, snapshot_id: u64) {
    let topics = (symbol_short!("v1_snp_dl"), snapshot_id);
    env.events().publish(topics, ());
}

pub fn snapshot_compared(env: &Env, snapshot_a: u64, snapshot_b: u64, identical: bool) {
    let topics = (symbol_short!("v1_snp_cp"), snapshot_a);
    env.events().publish(topics, (snapshot_b, identical));
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
