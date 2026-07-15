use soroban_sdk::{symbol_short, Address, Env};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
