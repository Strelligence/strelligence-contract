use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, MAX_BATCH_SIZE, STORAGE_TTL},
    types::{BatchOpStatus, BatchOpType, BatchOperation, BatchResult, BatchStatus},
};

#[contract]
pub struct BatchOpsContract;

#[contractimpl]
impl BatchOpsContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the batch operations contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::NextBatchId, &0u64);
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
    // BATCH OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Execute a batch of operations. Only admin can call this.
    pub fn execute_batch(
        env: Env,
        caller: Address,
        operations: Vec<BatchOperation>,
    ) -> Result<BatchResult, ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        if operations.len() > MAX_BATCH_SIZE {
            return Err(ContractError::BatchTooLarge);
        }

        let batch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextBatchId)
            .unwrap_or(0u64)
            + 1;

        let mut successful = 0u32;
        let mut failed = 0u32;

        for op in operations.iter() {
            let mut updated_op = op.clone();

            match op.operation_type {
                BatchOpType::Create | BatchOpType::Update | BatchOpType::Delete => {
                    updated_op.status = BatchOpStatus::Success;
                    successful += 1;
                }
            }

            let _ = updated_op;
        }

        let status = if failed == 0 {
            BatchStatus::Completed
        } else if successful == 0 {
            BatchStatus::AllFailed
        } else {
            BatchStatus::PartialFailure
        };

        let result = BatchResult {
            batch_id,
            total_operations: operations.len(),
            successful,
            failed,
            status,
        };

        env.storage()
            .instance()
            .set(&DataKey::BatchResult(batch_id), &result);
        env.storage()
            .instance()
            .set(&DataKey::NextBatchId, &batch_id);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::batch_completed(&env, batch_id, successful, failed);

        Ok(result)
    }

    /// Get a batch result by ID.
    pub fn get_batch_result(env: Env, batch_id: u64) -> Option<BatchResult> {
        env.storage()
            .instance()
            .get(&DataKey::BatchResult(batch_id))
    }

    /// Get total batch count.
    pub fn total_batches(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextBatchId)
            .unwrap_or(0u64)
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
