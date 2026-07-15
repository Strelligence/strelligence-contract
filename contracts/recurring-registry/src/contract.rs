use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, DEFAULT_PAGE_SIZE, INITIAL_VERSION, MAX_PAGE_SIZE, STORAGE_TTL},
    types::{Frequency, Subscription, SubscriptionStatus, SubscriptionType},
};

#[contract]
pub struct RecurringRegistryContract;

#[contractimpl]
impl RecurringRegistryContract {
    // ─────────────────────────────────────────────────────────────────────────
    // WRITE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Register a new recurring payment.
    ///
    /// Called by the backend after detecting a recurring pattern, or manually
    /// by the wallet owner. Returns the new subscription ID.
    pub fn create_subscription(
        env: Env,
        caller: Address,
        owner: Address,
        merchant: String,
        merchant_address: Option<Address>,
        frequency: Frequency,
        subscription_type: SubscriptionType,
        asset_code: String,
        asset_issuer: String,
        amount: i128,
        next_payment_ledger: u64,
        auto_detected: bool,
        custom_label: Option<String>,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Auto-increment ID
        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextSubscriptionId)
            .unwrap_or(0u64)
            + 1;
        env.storage()
            .instance()
            .set(&DataKey::NextSubscriptionId, &id);

        let subscription = Subscription {
            id,
            owner: owner.clone(),
            merchant,
            merchant_address,
            frequency,
            subscription_type,
            asset_code,
            asset_issuer,
            amount,
            active: true,
            status: SubscriptionStatus::Active,
            next_payment_ledger,
            last_payment_ledger: 0,
            created_at_ledger: env.ledger().sequence() as u64,
            auto_detected,
            custom_label,
        };

        // Persist the subscription record
        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &subscription);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Subscription(id), 0, STORAGE_TTL);

        // Update the wallet → [ids] index
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletSubscriptions(owner.clone()))
            .unwrap_or(Vec::new(&env));
        ids.push_back(id);
        env.storage()
            .persistent()
            .set(&DataKey::WalletSubscriptions(owner.clone()), &ids);
        env.storage().persistent().extend_ttl(
            &DataKey::WalletSubscriptions(owner.clone()),
            0,
            STORAGE_TTL,
        );

        events::subscription_created(&env, &owner, id);

        Ok(id)
    }

    /// Update mutable fields of an existing subscription.
    /// Only the subscription owner can call this.
    pub fn update_subscription(
        env: Env,
        caller: Address,
        id: u64,
        merchant: Option<String>,
        frequency: Option<Frequency>,
        amount: Option<i128>,
        next_payment_ledger: Option<u64>,
        custom_label: Option<String>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(ContractError::SubscriptionNotFound)?;

        // Only the owner can update
        sub.owner.require_auth();

        if let Some(m) = merchant {
            sub.merchant = m;
        }
        if let Some(f) = frequency {
            sub.frequency = f;
        }
        if let Some(a) = amount {
            if a <= 0 {
                return Err(ContractError::InvalidAmount);
            }
            sub.amount = a;
        }
        if let Some(n) = next_payment_ledger {
            sub.next_payment_ledger = n;
        }
        if let Some(l) = custom_label {
            sub.custom_label = Some(l);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Subscription(id), 0, STORAGE_TTL);

        events::subscription_updated(&env, &sub.owner, id);

        Ok(())
    }

    /// Cancel a subscription permanently.
    /// Only the subscription owner can cancel.
    pub fn cancel_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(ContractError::SubscriptionNotFound)?;

        sub.owner.require_auth();

        if sub.status == SubscriptionStatus::Cancelled {
            return Err(ContractError::AlreadyCancelled);
        }

        sub.status = SubscriptionStatus::Cancelled;
        sub.active = false;

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);

        events::subscription_cancelled(&env, &sub.owner, id);

        Ok(())
    }

    /// Pause a subscription temporarily.
    pub fn pause_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(ContractError::SubscriptionNotFound)?;

        sub.owner.require_auth();

        if sub.status == SubscriptionStatus::Paused {
            return Err(ContractError::AlreadyInState);
        }
        if sub.status == SubscriptionStatus::Cancelled {
            return Err(ContractError::AlreadyCancelled);
        }

        sub.status = SubscriptionStatus::Paused;
        sub.active = false;

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);

        events::subscription_paused(&env, &sub.owner, id);

        Ok(())
    }

    /// Resume a paused subscription.
    pub fn resume_subscription(env: Env, caller: Address, id: u64) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(ContractError::SubscriptionNotFound)?;

        sub.owner.require_auth();

        if sub.status == SubscriptionStatus::Cancelled {
            return Err(ContractError::AlreadyCancelled);
        }

        sub.status = SubscriptionStatus::Active;
        sub.active = true;

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);

        events::subscription_updated(&env, &sub.owner, id);

        Ok(())
    }

    /// Backend calls this after confirming a payment occurred.
    /// Advances the ledger pointers on the subscription record.
    pub fn confirm_payment(
        env: Env,
        caller: Address,
        id: u64,
        payment_ledger: u64,
        next_payment_ledger: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&DataKey::Subscription(id))
            .ok_or(ContractError::SubscriptionNotFound)?;

        if sub.status == SubscriptionStatus::Cancelled || sub.status == SubscriptionStatus::Expired
        {
            return Err(ContractError::InactiveSubscription);
        }

        sub.last_payment_ledger = payment_ledger;
        sub.next_payment_ledger = next_payment_ledger;

        env.storage()
            .persistent()
            .set(&DataKey::Subscription(id), &sub);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Subscription(id), 0, STORAGE_TTL);

        events::payment_confirmed(&env, &sub.owner, id);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UPGRADE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the contract with an admin for upgrade management.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInState);
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

    /// Fetch a single subscription by its ID.
    pub fn get_subscription(env: Env, id: u64) -> Option<Subscription> {
        env.storage().persistent().get(&DataKey::Subscription(id))
    }

    /// List subscription IDs registered for a wallet with pagination.
    pub fn list_wallet_subscriptions(
        env: Env,
        owner: Address,
        offset: u32,
        limit: u32,
    ) -> Vec<u64> {
        let limit = limit.min(MAX_PAGE_SIZE).max(1);
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let start = offset as usize;
        let end = (start + limit as usize).min(ids.len() as usize);

        let mut result = Vec::new(&env);
        for i in start..end {
            result.push_back(ids.get_unchecked(i as u32));
        }
        result
    }

    /// List only active subscriptions for a wallet (full objects) with pagination.
    pub fn list_active_subscriptions(
        env: Env,
        owner: Address,
        offset: u32,
        limit: u32,
    ) -> Vec<Subscription> {
        let limit = limit.min(MAX_PAGE_SIZE).max(1);
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let mut active = Vec::new(&env);
        let mut count = 0u32;
        let mut skipped = 0u32;

        for id in ids.iter() {
            if let Some(sub) = env
                .storage()
                .persistent()
                .get::<_, Subscription>(&DataKey::Subscription(id))
            {
                if sub.status == SubscriptionStatus::Active {
                    if skipped < offset {
                        skipped += 1;
                    } else if count < limit {
                        active.push_back(sub);
                        count += 1;
                    }
                }
            }
        }
        active
    }

    /// Total subscriptions ever created (global counter).
    pub fn total_subscriptions(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextSubscriptionId)
            .unwrap_or(0u64)
    }
}
