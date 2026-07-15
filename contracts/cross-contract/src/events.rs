use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn cross_call_initiated(env: &Env, caller: &Address, call_id: u64) {
    let topics = (symbol_short!("v1_cc_ini"), caller.clone());
    env.events().publish(topics, call_id);
}

pub fn cross_call_completed(env: &Env, call_id: u64, success: bool) {
    let topics = (symbol_short!("v1_cc_cmp"), call_id);
    env.events().publish(topics, success);
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
