use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, STORAGE_TTL},
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
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::WalletSubscriptions(owner.clone()), 0, STORAGE_TTL);

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
    pub fn cancel_subscription(
        env: Env,
        caller: Address,
        id: u64,
    ) -> Result<(), ContractError> {
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
    pub fn pause_subscription(
        env: Env,
        caller: Address,
        id: u64,
    ) -> Result<(), ContractError> {
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
    pub fn resume_subscription(
        env: Env,
        caller: Address,
        id: u64,
    ) -> Result<(), ContractError> {
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

        if sub.status == SubscriptionStatus::Cancelled
            || sub.status == SubscriptionStatus::Expired
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
    // READ FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Fetch a single subscription by its ID.
    pub fn get_subscription(env: Env, id: u64) -> Option<Subscription> {
        env.storage()
            .persistent()
            .get(&DataKey::Subscription(id))
    }

    /// List all subscription IDs registered for a wallet.
    pub fn list_wallet_subscriptions(env: Env, owner: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::WalletSubscriptions(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// List only active subscriptions for a wallet (full objects).
    pub fn list_active_subscriptions(env: Env, owner: Address) -> Vec<Subscription> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletSubscriptions(owner))
            .unwrap_or(Vec::new(&env));

        let mut active = Vec::new(&env);
        for id in ids.iter() {
            if let Some(sub) = env
                .storage()
                .persistent()
                .get::<_, Subscription>(&DataKey::Subscription(id))
            {
                if sub.status == SubscriptionStatus::Active {
                    active.push_back(sub);
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