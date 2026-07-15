use soroban_sdk::{symbol_short, Address, Env};

pub const EVENT_VERSION: u32 = 1;

pub fn role_granted(env: &Env, admin: &Address, grantee: &Address, role: u32) {
    let topics = (symbol_short!("v1_rolg"), admin.clone());
    env.events().publish(topics, (grantee.clone(), role));
}

pub fn role_revoked(env: &Env, admin: &Address, grantee: &Address) {
    let topics = (symbol_short!("v1_rolr"), admin.clone());
    env.events().publish(topics, grantee.clone());
}

pub fn permission_checked(env: &Env, address: &Address, allowed: bool) {
    let topics = (symbol_short!("v1_permck"), address.clone());
    env.events().publish(topics, allowed);
}

pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
