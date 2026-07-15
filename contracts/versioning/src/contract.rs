use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{CompatibilityCheck, SemanticVersion, VersionRecord},
};

#[contract]
pub struct VersioningContract;

#[contractimpl]
impl VersioningContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the versioning contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextRecordId, &0u64);
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
    // VERSION MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Register a new version. Only admin can call this.
    pub fn register_version(
        env: Env,
        caller: Address,
        contract_address: Address,
        major: u32,
        minor: u32,
        patch: u32,
        wasm_hash: Bytes,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextRecordId)
            .unwrap_or(0u64)
            + 1;

        let version = SemanticVersion {
            major,
            minor,
            patch,
        };

        let record = VersionRecord {
            id,
            contract_address: contract_address.clone(),
            version,
            wasm_hash,
            created_at_ledger: env.ledger().sequence() as u64,
            deprecated: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::VersionRecord(id), &record);
        env.storage().instance().set(&DataKey::NextRecordId, &id);
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion(contract_address), &id);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::version_registered(&env, &admin, id);

        Ok(id)
    }

    /// Deprecate a version. Only admin can call this.
    pub fn deprecate_version(
        env: Env,
        caller: Address,
        record_id: u64,
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

        let mut record: VersionRecord = env
            .storage()
            .instance()
            .get(&DataKey::VersionRecord(record_id))
            .ok_or(ContractError::VersionNotFound)?;

        record.deprecated = true;

        env.storage()
            .instance()
            .set(&DataKey::VersionRecord(record_id), &record);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::version_deprecated(&env, record_id);

        Ok(())
    }

    /// Check compatibility between two versions.
    pub fn check_compatibility(
        env: Env,
        from_major: u32,
        from_minor: u32,
        from_patch: u32,
        to_major: u32,
        to_minor: u32,
        to_patch: u32,
    ) -> CompatibilityCheck {
        let from_version = SemanticVersion {
            major: from_major,
            minor: from_minor,
            patch: from_patch,
        };

        let to_version = SemanticVersion {
            major: to_major,
            minor: to_minor,
            patch: to_patch,
        };

        let breaking_changes = to_major != from_major;
        let compatible = if breaking_changes {
            false
        } else if to_major == from_major && to_minor == from_minor {
            true
        } else {
            to_major == from_major
        };

        CompatibilityCheck {
            from_version,
            to_version,
            compatible,
            breaking_changes,
        }
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

    /// Get a version record by ID.
    pub fn get_version_record(env: Env, record_id: u64) -> Option<VersionRecord> {
        env.storage()
            .instance()
            .get(&DataKey::VersionRecord(record_id))
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

    /// Get total version records.
    pub fn total_records(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextRecordId)
            .unwrap_or(0u64)
    }
}
