use soroban_sdk::{symbol_short, Address, Env, String};

/// Event version for schema evolution
pub const EVENT_VERSION: u32 = 1;

/// Emitted when metadata is first attached to a transaction.
/// Topics: ("v1_meta_a", owner_address)
/// Data:   tx_hash (String)
pub fn metadata_added(env: &Env, owner: &Address, tx_hash: &String) {
    let topics = (symbol_short!("v1_meta_a"), owner.clone());
    env.events().publish(topics, tx_hash.clone());
}

/// Emitted when existing metadata is updated by the owner.
/// Topics: ("v1_meta_u", owner_address)
/// Data:   tx_hash (String)
pub fn metadata_updated(env: &Env, owner: &Address, tx_hash: &String) {
    let topics = (symbol_short!("v1_meta_u"), owner.clone());
    env.events().publish(topics, tx_hash.clone());
}

/// Emitted when the contract is upgraded to a new WASM hash.
/// Topics: ("v1_upgrd", admin_address)
/// Data:   new_version (u32)
pub fn contract_upgraded(env: &Env, admin: &Address, new_version: u32) {
    let topics = (symbol_short!("v1_upgrd"), admin.clone());
    env.events().publish(topics, new_version);
}
