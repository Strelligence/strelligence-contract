use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, DEFAULT_BATCH_SIZE, INITIAL_VERSION, STORAGE_TTL},
    types::{GasProfile, StorageOptimization},
};

#[contract]
pub struct GasOptimizerContract;

#[contractimpl]
impl GasOptimizerContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the gas optimizer with default settings.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let settings = StorageOptimization {
            batch_size: DEFAULT_BATCH_SIZE,
            compress: true,
            cache_enabled: true,
        };

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::OptimizationSettings, &settings);
        env.storage()
            .instance()
            .set(&DataKey::NextProfileId, &0u64);
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
    // GAS PROFILING
    // ─────────────────────────────────────────────────────────────────────────

    /// Record a gas profile for an operation.
    pub fn record_gas_profile(
        env: Env,
        caller: Address,
        operation: u32,
        read_count: u32,
        write_count: u32,
        bytes_read: u32,
        bytes_written: u32,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::Unauthorized);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProfileId)
            .unwrap_or(0u64)
            + 1;

        let profile = GasProfile {
            operation,
            read_count,
            write_count,
            bytes_read,
            bytes_written,
        };

        env.storage()
            .instance()
            .set(&DataKey::GasProfile(id), &profile);
        env.storage()
            .instance()
            .set(&DataKey::NextProfileId, &id);

        env.storage().instance().extend_ttl(0, 6_312_000);

        let gas_used = (read_count + write_count) as u64;
        events::gas_profile_recorded(&env, operation, gas_used);

        Ok(id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // OPTIMIZATION SETTINGS
    // ─────────────────────────────────────────────────────────────────────────

    /// Update optimization settings. Only admin can call this.
    pub fn update_settings(
        env: Env,
        caller: Address,
        batch_size: u32,
        compress: bool,
        cache_enabled: bool,
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

        if batch_size == 0 || batch_size > 1000 {
            return Err(ContractError::InvalidBatchSize);
        }

        let settings = StorageOptimization {
            batch_size,
            compress,
            cache_enabled,
        };

        env.storage()
            .instance()
            .set(&DataKey::OptimizationSettings, &settings);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::optimization_applied(&env, &admin, batch_size);

        Ok(())
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

    /// Get a gas profile by ID.
    pub fn get_gas_profile(env: Env, profile_id: u64) -> Option<GasProfile> {
        env.storage().instance().get(&DataKey::GasProfile(profile_id))
    }

    /// Get optimization settings.
    pub fn get_settings(env: Env) -> Option<StorageOptimization> {
        env.storage()
            .instance()
            .get(&DataKey::OptimizationSettings)
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

    /// Get the total number of gas profiles.
    pub fn total_profiles(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextProfileId)
            .unwrap_or(0u64)
    }
}
