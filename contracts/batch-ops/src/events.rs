use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn batch_completed(env: &Env, batch_id: u64, successful: u32, failed: u32) {
    let topics = (symbol_short!("v1_batcmp"), batch_id);
    env.events().publish(topics, (successful, failed));
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
