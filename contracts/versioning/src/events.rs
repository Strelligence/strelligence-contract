use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn version_registered(env: &Env, admin: &Address, record_id: u64) {
    let topics = (symbol_short!("v1_ver_rg"), admin.clone());
    env.events().publish(topics, record_id);
}

pub fn version_deprecated(env: &Env, record_id: u64) {
    let topics = (symbol_short!("v1_ver_dp"), record_id);
    env.events().publish(topics, ());
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
