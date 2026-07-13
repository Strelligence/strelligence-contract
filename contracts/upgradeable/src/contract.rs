use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, IntoVal};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION},
};

#[contract]
pub struct UpgradeableContract;

#[contractimpl]
impl UpgradeableContract {
    /// Initialize the upgradeable contract with an admin.
    pub fn initialize(env: Env, admin: Address) {
        if env
            .storage()
            .instance()
            .has(&DataKey::Admin)
        {
            panic!("already initialized");
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Version, &INITIAL_VERSION);
        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &placeholder);

        env.storage()
            .instance()
            .extend_ttl(0, 6312000);
    }

    /// Upgrade the contract to a new WASM hash. Only the admin can call this.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: Bytes) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::Unauthorized);
        }

        let current_hash: Bytes = env
            .storage()
            .instance()
            .get(&DataKey::WasmHash)
            .ok_or(ContractError::NotInitialized)?;

        if current_hash == new_wasm_hash {
            return Err(ContractError::SameWasmHash);
        }

        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(INITIAL_VERSION);
        let new_version = version + 1;

        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &new_wasm_hash);
        env.storage()
            .instance()
            .set(&DataKey::Version, &new_version);

        events::contract_upgraded(&env, &admin, new_version);

        Ok(())
    }

    /// Get the current contract version.
    pub fn get_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(INITIAL_VERSION)
    }

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Get the current WASM hash stored on-chain.
    pub fn get_wasm_hash(env: Env) -> Option<Bytes> {
        env.storage().instance().get(&DataKey::WasmHash)
    }
}
