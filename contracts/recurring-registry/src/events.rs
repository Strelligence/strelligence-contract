use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a new subscription is created.
/// Topics: ("sub_created", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_created(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("sub_crtd"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription's fields are updated.
/// Topics: ("sub_updated", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_updated(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("sub_upd"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription is cancelled.
/// Topics: ("sub_cancelled", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_cancelled(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("sub_can"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when a subscription is paused.
/// Topics: ("sub_paused", owner_address)
/// Data:   subscription_id (u64)
pub fn subscription_paused(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("sub_psd"), owner.clone());
    env.events().publish(topics, id);
}

/// Emitted when the backend confirms a payment occurred.
/// Topics: ("pay_confirmed", owner_address)
/// Data:   subscription_id (u64)
pub fn payment_confirmed(env: &Env, owner: &Address, id: u64) {
    let topics = (symbol_short!("pay_conf"), owner.clone());
    env.events().publish(topics, id);
}
