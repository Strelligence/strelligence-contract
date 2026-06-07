use soroban_sdk::{symbol_short, Address, Env, String};

/// Emitted when metadata is first attached to a transaction.
/// Topics: ("meta_added", owner_address)
/// Data:   tx_hash (String)
pub fn metadata_added(env: &Env, owner: &Address, tx_hash: &String) {
    let topics = (symbol_short!("meta_add"), owner.clone());
    env.events().publish(topics, tx_hash.clone());
}

/// Emitted when existing metadata is updated by the owner.
/// Topics: ("meta_updated", owner_address)
/// Data:   tx_hash (String)
pub fn metadata_updated(env: &Env, owner: &Address, tx_hash: &String) {
    let topics = (symbol_short!("meta_upd"), owner.clone());
    env.events().publish(topics, tx_hash.clone());
}
