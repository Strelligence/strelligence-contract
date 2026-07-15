use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{Rule, RuleStatus, RuleTrigger, RuleType},
};

#[contract]
pub struct AutomationRulesContract;

#[contractimpl]
impl AutomationRulesContract {
    // ─────────────────────────────────────────────────────────────────────────
    // WRITE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new automation rule.
    pub fn create_rule(
        env: Env,
        caller: Address,
        rule_type: RuleType,
        trigger: RuleTrigger,
        label: String,
        trigger_params: String,
        action_params: String,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextRuleId)
            .unwrap_or(0u64)
            + 1;
        env.storage().instance().set(&DataKey::NextRuleId, &id);

        let rule = Rule {
            id,
            owner: caller.clone(),
            rule_type,
            trigger,
            status: RuleStatus::Active,
            label,
            trigger_params,
            action_params,
            created_at_ledger: env.ledger().sequence() as u64,
            last_executed_ledger: 0,
            execution_count: 0,
        };

        env.storage().persistent().set(&DataKey::Rule(id), &rule);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Rule(id), 0, STORAGE_TTL);

        let mut rule_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletRules(caller.clone()))
            .unwrap_or(Vec::new(&env));
        rule_ids.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::WalletRules(caller.clone()), &rule_ids);
        env.storage().persistent().extend_ttl(
            &DataKey::WalletRules(caller.clone()),
            0,
            STORAGE_TTL,
        );

        events::rule_created(&env, &caller, id);

        Ok(id)
    }

    /// Update mutable fields of an existing rule. Only the owner can call this.
    pub fn update_rule(
        env: Env,
        caller: Address,
        id: u64,
        label: Option<String>,
        trigger_params: Option<String>,
        action_params: Option<String>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut rule: Rule = env
            .storage()
            .persistent()
            .get(&DataKey::Rule(id))
            .ok_or(ContractError::RuleNotFound)?;

        rule.owner.require_auth();

        if let Some(l) = label {
            rule.label = l;
        }
        if let Some(tp) = trigger_params {
            rule.trigger_params = tp;
        }
        if let Some(ap) = action_params {
            rule.action_params = ap;
        }

        env.storage().persistent().set(&DataKey::Rule(id), &rule);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Rule(id), 0, STORAGE_TTL);

        events::rule_updated(&env, &rule.owner, id);

        Ok(())
    }

    /// Pause an active rule.
    pub fn pause_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut rule: Rule = env
            .storage()
            .persistent()
            .get(&DataKey::Rule(id))
            .ok_or(ContractError::RuleNotFound)?;

        rule.owner.require_auth();

        if rule.status == RuleStatus::Deleted {
            return Err(ContractError::AlreadyDeleted);
        }

        rule.status = RuleStatus::Paused;

        env.storage().persistent().set(&DataKey::Rule(id), &rule);

        events::rule_paused(&env, &rule.owner, id);

        Ok(())
    }

    /// Resume a paused rule.
    pub fn resume_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut rule: Rule = env
            .storage()
            .persistent()
            .get(&DataKey::Rule(id))
            .ok_or(ContractError::RuleNotFound)?;

        rule.owner.require_auth();

        if rule.status == RuleStatus::Deleted {
            return Err(ContractError::AlreadyDeleted);
        }

        rule.status = RuleStatus::Active;

        env.storage().persistent().set(&DataKey::Rule(id), &rule);

        events::rule_updated(&env, &rule.owner, id);

        Ok(())
    }

    /// Soft-delete a rule. Sets status to Deleted.
    pub fn delete_rule(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut rule: Rule = env
            .storage()
            .persistent()
            .get(&DataKey::Rule(id))
            .ok_or(ContractError::RuleNotFound)?;

        rule.owner.require_auth();

        if rule.status == RuleStatus::Deleted {
            return Err(ContractError::AlreadyDeleted);
        }

        rule.status = RuleStatus::Deleted;

        env.storage().persistent().set(&DataKey::Rule(id), &rule);

        events::rule_deleted(&env, &rule.owner, id);

        Ok(())
    }

    /// Record that a rule was executed. Backend calls this after execution.
    pub fn record_execution(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut rule: Rule = env
            .storage()
            .persistent()
            .get(&DataKey::Rule(id))
            .ok_or(ContractError::RuleNotFound)?;

        rule.execution_count += 1;
        rule.last_executed_ledger = env.ledger().sequence() as u64;

        env.storage().persistent().set(&DataKey::Rule(id), &rule);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Rule(id), 0, STORAGE_TTL);

        events::rule_executed(&env, &rule.owner, id);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UPGRADE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the contract with an admin for upgrade management.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Version, &INITIAL_VERSION);
        env.storage()
            .instance()
            .set(&DataKey::WasmHash, &placeholder);

        env.storage().instance().extend_ttl(0, 6_312_000);

        Ok(())
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

    // ─────────────────────────────────────────────────────────────────────────
    // READ FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Fetch a single rule by ID.
    pub fn get_rule(env: Env, id: u64) -> Option<Rule> {
        env.storage().persistent().get(&DataKey::Rule(id))
    }

    /// List all rule IDs for a wallet.
    pub fn list_wallet_rules(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::WalletRules(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// List only active rules for a wallet.
    pub fn list_active_rules(env: Env, owner: Address) -> Vec<Rule> {
        let rule_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletRules(owner))
            .unwrap_or(Vec::new(&env));

        let mut active = Vec::new(&env);
        for id in rule_ids.iter() {
            if let Some(rule) = env
                .storage()
                .persistent()
                .get::<_, Rule>(&DataKey::Rule(id))
            {
                if rule.status == RuleStatus::Active {
                    active.push_back(rule);
                }
            }
        }
        active
    }
}
