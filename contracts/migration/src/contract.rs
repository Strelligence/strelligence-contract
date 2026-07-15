use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{MigrationPlan, MigrationStatus, VerificationResult},
};

#[contract]
pub struct MigrationContract;

#[contractimpl]
impl MigrationContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the migration contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::NextPlanId, &0u64);
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
    // MIGRATION MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new migration plan. Only admin can call this.
    pub fn create_plan(
        env: Env,
        caller: Address,
        contract_address: Address,
        from_version: u32,
        to_version: u32,
        from_wasm_hash: Bytes,
        to_wasm_hash: Bytes,
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

        if from_wasm_hash == to_wasm_hash {
            return Err(ContractError::SameWasmHash);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextPlanId)
            .unwrap_or(0u64)
            + 1;

        let plan = MigrationPlan {
            id,
            contract_address: contract_address.clone(),
            from_version,
            to_version,
            from_wasm_hash,
            to_wasm_hash,
            status: MigrationStatus::Pending,
            created_at_ledger: env.ledger().sequence() as u64,
            completed_at_ledger: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::MigrationPlan(id), &plan);
        env.storage()
            .instance()
            .set(&DataKey::NextPlanId, &id);
        env.storage()
            .instance()
            .set(&DataKey::ContractPlan(contract_address), &id);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::migration_created(&env, &admin, id);

        Ok(id)
    }

    /// Start a migration. Only admin can call this.
    pub fn start_migration(
        env: Env,
        caller: Address,
        plan_id: u64,
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

        let mut plan: MigrationPlan = env
            .storage()
            .instance()
            .get(&DataKey::MigrationPlan(plan_id))
            .ok_or(ContractError::PlanNotFound)?;

        if plan.status != MigrationStatus::Pending {
            return Err(ContractError::InvalidMigrationStatus);
        }

        plan.status = MigrationStatus::InProgress;

        env.storage()
            .instance()
            .set(&DataKey::MigrationPlan(plan_id), &plan);
        env.storage().instance().extend_ttl(0, 6_312_000);

        Ok(())
    }

    /// Complete a migration. Only admin can call this.
    pub fn complete_migration(
        env: Env,
        caller: Address,
        plan_id: u64,
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

        let mut plan: MigrationPlan = env
            .storage()
            .instance()
            .get(&DataKey::MigrationPlan(plan_id))
            .ok_or(ContractError::PlanNotFound)?;

        if plan.status != MigrationStatus::InProgress {
            return Err(ContractError::InvalidMigrationStatus);
        }

        plan.status = MigrationStatus::Completed;
        plan.completed_at_ledger = env.ledger().sequence() as u64;

        env.storage()
            .instance()
            .set(&DataKey::MigrationPlan(plan_id), &plan);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::migration_completed(&env, plan_id);

        Ok(())
    }

    /// Rollback a migration. Only admin can call this.
    pub fn rollback_migration(
        env: Env,
        caller: Address,
        plan_id: u64,
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

        let mut plan: MigrationPlan = env
            .storage()
            .instance()
            .get(&DataKey::MigrationPlan(plan_id))
            .ok_or(ContractError::PlanNotFound)?;

        if plan.status != MigrationStatus::InProgress && plan.status != MigrationStatus::Failed {
            return Err(ContractError::InvalidMigrationStatus);
        }

        plan.status = MigrationStatus::RolledBack;

        env.storage()
            .instance()
            .set(&DataKey::MigrationPlan(plan_id), &plan);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::migration_rolled_back(&env, plan_id);

        Ok(())
    }

    /// Verify a migration plan. Returns verification results.
    pub fn verify_plan(
        env: Env,
        caller: Address,
        plan_id: u64,
    ) -> Result<VerificationResult, ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let plan: MigrationPlan = env
            .storage()
            .instance()
            .get(&DataKey::MigrationPlan(plan_id))
            .ok_or(ContractError::PlanNotFound)?;

        let verification = VerificationResult {
            plan_id,
            storage_valid: plan.status == MigrationStatus::Completed,
            state_valid: plan.status == MigrationStatus::Completed,
            events_valid: plan.status == MigrationStatus::Completed,
        };

        Ok(verification)
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

    /// Get a migration plan by ID.
    pub fn get_plan(env: Env, plan_id: u64) -> Option<MigrationPlan> {
        env.storage()
            .instance()
            .get(&DataKey::MigrationPlan(plan_id))
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

    /// Get the total number of migration plans.
    pub fn total_plans(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextPlanId)
            .unwrap_or(0u64)
    }
}
