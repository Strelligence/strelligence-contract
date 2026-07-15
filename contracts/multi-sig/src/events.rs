use soroban_sdk::{symbol_short, Address, Env, String, Vec};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

pub fn signer_added(env: &Env, admin: &Address, signer: &Address) {
    let topics = (symbol_short!("v1_sigadd"), admin.clone());
    env.events().publish(topics, signer.clone());
}

pub fn signer_removed(env: &Env, admin: &Address, signer: &Address) {
    let topics = (symbol_short!("v1_sigrem"), admin.clone());
    env.events().publish(topics, signer.clone());
}

pub fn threshold_updated(env: &Env, admin: &Address, old_threshold: u32, new_threshold: u32) {
    let topics = (symbol_short!("v1_thr_up"), admin.clone());
    env.events().publish(topics, (old_threshold, new_threshold));
}

pub fn proposal_created(env: &Env, proposer: &Address, proposal_id: u64) {
    let topics = (symbol_short!("v1_prl_cr"), proposer.clone());
    env.events().publish(topics, proposal_id);
}

pub fn proposal_signed(env: &Env, signer: &Address, proposal_id: u64) {
    let topics = (symbol_short!("v1_prl_sg"), signer.clone());
    env.events().publish(topics, proposal_id);
}

pub fn proposal_executed(env: &Env, proposal_id: u64) {
    let topics = (symbol_short!("v1_prl_ex"), proposal_id);
    env.events().publish(topics, ());
}

pub fn proposal_cancelled(env: &Env, proposal_id: u64) {
    let topics = (symbol_short!("v1_prl_cn"), proposal_id);
    env.events().publish(topics, ());
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
