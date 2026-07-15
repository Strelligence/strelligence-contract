use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{PauseHistory, PauseState},
};

fn default_admin(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    ))
}

#[contract]
pub struct EmergencyPauseContract;

#[contractimpl]
impl EmergencyPauseContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the emergency pause contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);

        let pause_state = PauseState {
            paused: false,
            paused_by: admin.clone(),
            paused_at_ledger: 0,
            unpause_at_ledger: None,
            reason: String::from_str(&env, ""),
        };

        let pause_history = PauseHistory {
            pause_count: 0,
            last_paused_at: 0,
            last_unpaused_at: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::PauseState, &pause_state);
        env.storage()
            .instance()
            .set(&DataKey::PauseHistory, &pause_history);
        env.storage()
            .instance()
            .set(&DataKey::Version, &INITIAL_VERSION);
        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &placeholder);

        env.storage().instance().extend_ttl(0, 6_312_000);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PAUSE MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Pause the contract. Only admin can call this.
    pub fn pause(
        env: Env,
        caller: Address,
        reason: String,
        unpause_at_ledger: Option<u64>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let mut pause_state: PauseState = env
            .storage()
            .instance()
            .get(&DataKey::PauseState)
            .ok_or(ContractError::NotInitialized)?;

        if pause_state.paused {
            return Err(ContractError::AlreadyPaused);
        }

        let mut history: PauseHistory = env
            .storage()
            .instance()
            .get(&DataKey::PauseHistory)
            .unwrap_or(PauseHistory {
                pause_count: 0,
                last_paused_at: 0,
                last_unpaused_at: 0,
            });

        let current_ledger = env.ledger().sequence() as u64;

        pause_state.paused = true;
        pause_state.paused_by = caller.clone();
        pause_state.paused_at_ledger = current_ledger;
        pause_state.unpause_at_ledger = unpause_at_ledger;
        pause_state.reason = reason;

        history.pause_count += 1;
        history.last_paused_at = current_ledger;

        env.storage()
            .instance()
            .set(&DataKey::PauseState, &pause_state);
        env.storage()
            .instance()
            .set(&DataKey::PauseHistory, &history);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::contract_paused(&env, &admin, 0);

        Ok(())
    }

    /// Unpause the contract. Only admin can call this.
    pub fn unpause(env: Env, caller: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let mut pause_state: PauseState = env
            .storage()
            .instance()
            .get(&DataKey::PauseState)
            .ok_or(ContractError::NotInitialized)?;

        if !pause_state.paused {
            return Err(ContractError::NotPaused);
        }

        let mut history: PauseHistory = env
            .storage()
            .instance()
            .get(&DataKey::PauseHistory)
            .unwrap_or(PauseHistory {
                pause_count: 0,
                last_paused_at: 0,
                last_unpaused_at: 0,
            });

        let current_ledger = env.ledger().sequence() as u64;

        pause_state.paused = false;
        pause_state.paused_at_ledger = 0;
        pause_state.unpause_at_ledger = None;
        pause_state.reason = String::from_str(&env, "");

        history.last_unpaused_at = current_ledger;

        env.storage()
            .instance()
            .set(&DataKey::PauseState, &pause_state);
        env.storage()
            .instance()
            .set(&DataKey::PauseHistory, &history);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::contract_unpaused(&env, &admin);

        Ok(())
    }

    /// Check if the contract is paused.
    pub fn is_paused(env: Env) -> bool {
        let pause_state: PauseState = env
            .storage()
            .instance()
            .get(&DataKey::PauseState)
            .unwrap_or(PauseState {
                paused: false,
                paused_by: default_admin(&env),
                paused_at_ledger: 0,
                unpause_at_ledger: None,
                reason: String::from_str(&env, ""),
            });

        if pause_state.paused {
            if let Some(unpause_at) = pause_state.unpause_at_ledger {
                let current_ledger = env.ledger().sequence() as u64;
                if current_ledger >= unpause_at {
                    return false;
                }
            }
            return true;
        }

        false
    }

    /// Get the current pause state.
    pub fn get_pause_state(env: Env) -> PauseState {
        env.storage()
            .instance()
            .get(&DataKey::PauseState)
            .unwrap_or(PauseState {
                paused: false,
                paused_by: default_admin(&env),
                paused_at_ledger: 0,
                unpause_at_ledger: None,
                reason: String::from_str(&env, ""),
            })
    }

    /// Get pause history.
    pub fn get_pause_history(env: Env) -> PauseHistory {
        env.storage()
            .instance()
            .get(&DataKey::PauseHistory)
            .unwrap_or(PauseHistory {
                pause_count: 0,
                last_paused_at: 0,
                last_unpaused_at: 0,
            })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UPGRADE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Upgrade the contract to a new WASM hash. Only the admin can call this.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: Bytes) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
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

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::contract_upgraded(&env, &admin, new_version);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // READ FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Get the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    /// Get the current contract version.
    pub fn get_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(INITIAL_VERSION)
    }

    /// Get the current WASM hash stored on-chain.
    pub fn get_wasm_hash(env: Env) -> Option<Bytes> {
        env.storage().instance().get(&DataKey::WasmHash)
    }
}
