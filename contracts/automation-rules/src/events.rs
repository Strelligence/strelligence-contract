use soroban_sdk::{symbol_short, Address, Env};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

pub fn rule_created(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_rle_cr"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_updated(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_rle_up"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_paused(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_rle_ps"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_deleted(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_rle_dl"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_executed(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_rle_ex"), owner.clone());
    env.events().publish(topics, id);
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
