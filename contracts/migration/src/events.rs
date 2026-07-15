use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn migration_created(env: &Env, admin: &Address, plan_id: u64) {
    let topics = (symbol_short!("v1_mig_cr"), admin.clone());
    env.events().publish(topics, plan_id);
}

pub fn migration_completed(env: &Env, plan_id: u64) {
    let topics = (symbol_short!("v1_mig_cp"), plan_id);
    env.events().publish(topics, ());
}

pub fn migration_rolled_back(env: &Env, plan_id: u64) {
    let topics = (symbol_short!("v1_mig_rb"), plan_id);
    env.events().publish(topics, ());
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
