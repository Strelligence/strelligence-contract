use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn gas_profile_recorded(env: &Env, operation: u32, gas_used: u64) {
    let topics = (symbol_short!("v1_gaspr"), operation);
    env.events().publish(topics, gas_used);
}

pub fn optimization_applied(env: &Env, admin: &Address, optimization_type: u32) {
    let topics = (symbol_short!("v1_gasop"), admin.clone());
    env.events().publish(topics, optimization_type);
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
