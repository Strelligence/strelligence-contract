use soroban_sdk::{symbol_short, Address, Env};

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("upgraded"), admin.clone());
    env.events().publish(topics, new_version);
}
