use soroban_sdk::{symbol_short, Address, Env};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

/// Emitted when a new subscription is created.
/// Topics: ("v1_sub_cr", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_created(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_sub_cr"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription's fields are updated.
/// Topics: ("v1_sub_up", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_updated(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_sub_up"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription is cancelled.
/// Topics: ("v1_sub_cn", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_cancelled(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_sub_cn"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription is paused.
/// Topics: ("v1_sub_ps", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_paused(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_sub_ps"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when the backend confirms a payment occurred.
/// Topics: ("v1_pay_cf", owner_address)
/// Data:   subscription_id (u64)
pub fn payment_confirmed(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("v1_pay_cf"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when the contract is upgraded to a new WASM hash.
/// Topics: ("v1_upgrd", admin_address)
/// Data:   new_version (u32)
pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
