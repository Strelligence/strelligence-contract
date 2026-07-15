use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn event_indexed(env: &Env, event_id: u64) {
    let topics = (symbol_short!("v1_evtidx"), event_id);
    env.events().publish(topics, ());
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
