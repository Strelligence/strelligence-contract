use soroban_sdk::{symbol_short, Address, Env};

pub fn rule_created(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("rule_cr"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_updated(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("rule_up"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_paused(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("rule_ps"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_deleted(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("rule_dl"), owner.clone());
    env.events().publish(topics, id);
}

pub fn rule_executed(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("rule_ex"), owner.clone());
    env.events().publish(topics, id);
}
