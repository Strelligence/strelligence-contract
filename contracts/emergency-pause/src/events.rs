use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn contract_paused(env: &Env, admin: &Address, reason: u32) {
    let topics = (symbol_short!("v1_pseon"), admin.clone());
    env.events().publish(topics, reason);
}

pub fn contract_unpaused(env: &Env, admin: &Address) {
    let topics = (symbol_short!("v1_pseof"), admin.clone());
    env.events().publish(topics, ());
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
