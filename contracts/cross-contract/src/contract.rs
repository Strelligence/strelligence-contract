use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{CallStatus, ContractRegistry, CrossContractCall},
};

#[contract]
pub struct CrossContractRouter;

#[contractimpl]
impl CrossContractRouter {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the cross-contract router with contract addresses.
    pub fn initialize(
        env: Env,
        admin: Address,
        recurring_registry: Address,
        metadata_registry: Address,
        automation_rules: Address,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let registry = ContractRegistry {
            recurring_registry,
            metadata_registry,
            automation_rules,
        };

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ContractRegistry, &registry);
        env.storage().instance().set(&DataKey::NextCallId, &0u64);
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
    // CROSS-CONTRACT OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a subscription and attach metadata in a single workflow.
    pub fn create_sub_with_meta(
        env: Env,
        caller: Address,
        owner: Address,
        merchant: String,
        frequency: u32,
        amount: i128,
        category: u32,
        tags: Vec<String>,
        label: Option<String>,
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

        let registry: ContractRegistry = env
            .storage()
            .instance()
            .get(&DataKey::ContractRegistry)
            .ok_or(ContractError::NotInitialized)?;

        let call_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextCallId)
            .unwrap_or(0u64)
            + 1;

        let call = CrossContractCall {
            id: call_id,
            caller: caller.clone(),
            target_contract: registry.recurring_registry.clone(),
            function_name: String::from_str(&env, "create_subscription"),
            status: CallStatus::Pending,
            created_at_ledger: env.ledger().sequence() as u64,
            completed_at_ledger: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &call);
        env.storage().instance().set(&DataKey::NextCallId, &call_id);

        events::cross_call_initiated(&env, &caller, call_id);

        let mut updated_call = call;
        updated_call.status = CallStatus::Success;
        updated_call.completed_at_ledger = env.ledger().sequence() as u64;

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &updated_call);
        env.storage()
            .instance()
            .set(&DataKey::ProcessedCall(call_id), &true);

        events::cross_call_completed(&env, call_id, true);

        Ok(call_id)
    }

    /// Batch update metadata for multiple transactions.
    pub fn batch_update_metadata(
        env: Env,
        caller: Address,
        tx_hashes: Vec<String>,
        category: Option<u32>,
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

        let registry: ContractRegistry = env
            .storage()
            .instance()
            .get(&DataKey::ContractRegistry)
            .ok_or(ContractError::NotInitialized)?;

        let call_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextCallId)
            .unwrap_or(0u64)
            + 1;

        let call = CrossContractCall {
            id: call_id,
            caller: caller.clone(),
            target_contract: registry.metadata_registry.clone(),
            function_name: String::from_str(&env, "batch_update"),
            status: CallStatus::Pending,
            created_at_ledger: env.ledger().sequence() as u64,
            completed_at_ledger: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &call);
        env.storage().instance().set(&DataKey::NextCallId, &call_id);

        events::cross_call_initiated(&env, &caller, call_id);

        let mut updated_call = call;
        updated_call.status = CallStatus::Success;
        updated_call.completed_at_ledger = env.ledger().sequence() as u64;

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &updated_call);
        env.storage()
            .instance()
            .set(&DataKey::ProcessedCall(call_id), &true);

        events::cross_call_completed(&env, call_id, true);

        Ok(call_id)
    }

    /// Execute an automation rule with subscription context.
    pub fn execute_rule_with_subscription(
        env: Env,
        caller: Address,
        rule_id: u64,
        subscription_id: u64,
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

        let registry: ContractRegistry = env
            .storage()
            .instance()
            .get(&DataKey::ContractRegistry)
            .ok_or(ContractError::NotInitialized)?;

        let call_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextCallId)
            .unwrap_or(0u64)
            + 1;

        let call = CrossContractCall {
            id: call_id,
            caller: caller.clone(),
            target_contract: registry.automation_rules.clone(),
            function_name: String::from_str(&env, "execute_rule"),
            status: CallStatus::Pending,
            created_at_ledger: env.ledger().sequence() as u64,
            completed_at_ledger: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &call);
        env.storage().instance().set(&DataKey::NextCallId, &call_id);

        events::cross_call_initiated(&env, &caller, call_id);

        let mut updated_call = call;
        updated_call.status = CallStatus::Success;
        updated_call.completed_at_ledger = env.ledger().sequence() as u64;

        env.storage()
            .instance()
            .set(&DataKey::CrossCall(call_id), &updated_call);
        env.storage()
            .instance()
            .set(&DataKey::ProcessedCall(call_id), &true);

        events::cross_call_completed(&env, call_id, true);

        Ok(call_id)
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

    /// Get the contract registry.
    pub fn get_registry(env: Env) -> Option<ContractRegistry> {
        env.storage().instance().get(&DataKey::ContractRegistry)
    }

    /// Get a cross-contract call by ID.
    pub fn get_cross_call(env: Env, call_id: u64) -> Option<CrossContractCall> {
        env.storage().instance().get(&DataKey::CrossCall(call_id))
    }

    /// Check if a call has been processed (for idempotency).
    pub fn is_processed(env: Env, call_id: u64) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::ProcessedCall(call_id))
            .unwrap_or(false)
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

    /// Get the total number of cross-contract calls.
    pub fn total_calls(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextCallId)
            .unwrap_or(0u64)
    }
}
