use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{Snapshot, SnapshotDiff},
};

#[contract]
pub struct SnapshotsContract;

#[contractimpl]
impl SnapshotsContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the snapshots contract with an admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::NextSnapshotId, &0u64);
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
    // SNAPSHOT MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new snapshot. Only the admin can call this.
    pub fn create_snapshot(
        env: Env,
        caller: Address,
        contract_address: Address,
        label: String,
        state_hash: Bytes,
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

        if label.is_empty() {
            return Err(ContractError::InvalidLabel);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextSnapshotId)
            .unwrap_or(0u64)
            + 1;

        let snapshot = Snapshot {
            id,
            creator: caller.clone(),
            contract_address: contract_address.clone(),
            label,
            state_hash,
            created_at_ledger: env.ledger().sequence() as u64,
            expired: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Snapshot(id), &snapshot);
        env.storage().instance().set(&DataKey::NextSnapshotId, &id);

        let mut contract_snapshots: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ContractSnapshots(contract_address.clone()))
            .unwrap_or(Vec::new(&env));
        contract_snapshots.push_back(id);
        env.storage().persistent().set(
            &DataKey::ContractSnapshots(contract_address),
            &contract_snapshots,
        );

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::snapshot_created(&env, &caller, id);

        Ok(id)
    }

    /// Delete a snapshot (mark as expired). Only the admin can call this.
    pub fn delete_snapshot(
        env: Env,
        caller: Address,
        snapshot_id: u64,
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

        let mut snapshot: Snapshot = env
            .storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_id))
            .ok_or(ContractError::SnapshotNotFound)?;

        snapshot.expired = true;

        env.storage()
            .instance()
            .set(&DataKey::Snapshot(snapshot_id), &snapshot);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::snapshot_deleted(&env, snapshot_id);

        Ok(())
    }

    /// Compare two snapshots. Returns a diff summary.
    pub fn compare_snapshots(
        env: Env,
        snapshot_a_id: u64,
        snapshot_b_id: u64,
    ) -> Result<SnapshotDiff, ContractError> {
        let snapshot_a: Snapshot = env
            .storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_a_id))
            .ok_or(ContractError::SnapshotNotFound)?;

        let snapshot_b: Snapshot = env
            .storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_b_id))
            .ok_or(ContractError::SnapshotNotFound)?;

        let identical = snapshot_a.state_hash == snapshot_b.state_hash;

        let diff = SnapshotDiff {
            snapshot_a_id,
            snapshot_b_id,
            identical,
            added_keys: 0,
            removed_keys: 0,
            modified_keys: if identical { 0 } else { 1 },
        };

        events::snapshot_compared(&env, snapshot_a_id, snapshot_b_id, identical);

        Ok(diff)
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

    /// Get a snapshot by ID.
    pub fn get_snapshot(env: Env, snapshot_id: u64) -> Option<Snapshot> {
        env.storage()
            .instance()
            .get(&DataKey::Snapshot(snapshot_id))
    }

    /// Get all snapshots for a contract.
    pub fn get_contract_snapshots(env: Env, contract_address: Address) -> Vec<Snapshot> {
        let snapshot_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ContractSnapshots(contract_address))
            .unwrap_or(Vec::new(&env));

        let mut snapshots = Vec::new(&env);
        for id in snapshot_ids.iter() {
            if let Some(snapshot) = env
                .storage()
                .instance()
                .get::<_, Snapshot>(&DataKey::Snapshot(id))
            {
                if !snapshot.expired {
                    snapshots.push_back(snapshot);
                }
            }
        }
        snapshots
    }

    /// Get the total number of snapshots.
    pub fn total_snapshots(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextSnapshotId)
            .unwrap_or(0u64)
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
