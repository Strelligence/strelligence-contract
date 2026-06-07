#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol, Vec, Map,
    log,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const SUB_KEY: Symbol = symbol_short!("SUB");
const WALLET_IDX: Symbol = symbol_short!("WIDX");
const SUB_COUNT: Symbol = symbol_short!("COUNT");

// ─── Data Types ──────────────────────────────────────────────────────────────

/// Frequency options for a recurring payment
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum Frequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Annually,
    Custom, // for irregular but detected patterns
}

/// Status of a subscription
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Cancelled,
    Expired,
}

/// Category/type of the recurring payment  
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum RecurringType {
    Subscription,   // e.g. Netflix, Spotify
    Payroll,        // outgoing salary payments
    Income,         // recurring incoming (salary, rent received)
    Savings,        // auto-save transfers
    Bill,           // utilities, rent
    Investment,     // DCA, staking top-ups
    Transfer,       // wallet-to-wallet recurring
    Other,
}

/// Core subscription / recurring payment record
#[contracttype]
#[derive(Clone)]
pub struct Subscription {
    pub id: u64,
    pub wallet: Address,
    pub merchant_name: String,
    pub merchant_address: Option<Address>, // may be unknown at detection time
    pub frequency: Frequency,
    pub recurring_type: RecurringType,
    pub asset_code: String,      // e.g. "USDC", "XLM"
    pub asset_issuer: String,    // issuer address or "native"
    pub amount: i128,            // in stroops / base units
    pub status: SubscriptionStatus,
    pub next_payment_ledger: u64,
    pub last_payment_ledger: u64,
    pub created_at_ledger: u64,
    pub detected_by_backend: bool, // true = auto-detected, false = manually added
    pub custom_label: Option<String>,
}

// ─── Events ──────────────────────────────────────────────────────────────────

fn emit_subscription_created(env: &Env, id: u64, wallet: &Address) {
    let topics = (symbol_short!("sub_new"), wallet.clone());
    env.events().publish(topics, id);
}

fn emit_subscription_updated(env: &Env, id: u64, wallet: &Address) {
    let topics = (symbol_short!("sub_upd"), wallet.clone());
    env.events().publish(topics, id);
}

fn emit_subscription_cancelled(env: &Env, id: u64, wallet: &Address) {
    let topics = (symbol_short!("sub_can"), wallet.clone());
    env.events().publish(topics, id);
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct RecurringRegistryContract;

#[contractimpl]
impl RecurringRegistryContract {

    // ── Write Functions ───────────────────────────────────────────────────────

    /// Register a new recurring payment / subscription.
    /// Called by the backend indexer after detection, or manually by the user.
    pub fn create_subscription(
        env: Env,
        caller: Address,
        wallet: Address,
        merchant_name: String,
        merchant_address: Option<Address>,
        frequency: Frequency,
        recurring_type: RecurringType,
        asset_code: String,
        asset_issuer: String,
        amount: i128,
        next_payment_ledger: u64,
        detected_by_backend: bool,
        custom_label: Option<String>,
    ) -> u64 {
        // Only the wallet owner or the authorized backend can register
        caller.require_auth();

        // Auto-increment ID
        let count: u64 = env
            .storage()
            .instance()
            .get(&SUB_COUNT)
            .unwrap_or(0u64);
        let new_id = count + 1;
        env.storage().instance().set(&SUB_COUNT, &new_id);

        let current_ledger = env.ledger().sequence() as u64;

        let sub = Subscription {
            id: new_id,
            wallet: wallet.clone(),
            merchant_name,
            merchant_address,
            frequency,
            recurring_type,
            asset_code,
            asset_issuer,
            amount,
            status: SubscriptionStatus::Active,
            next_payment_ledger,
            last_payment_ledger: 0,
            created_at_ledger: current_ledger,
            detected_by_backend,
            custom_label,
        };

        // Store the subscription by ID
        let sub_key = (SUB_KEY, new_id);
        env.storage().persistent().set(&sub_key, &sub);

        // Update the wallet → [subscription_ids] index
        let wallet_key = (WALLET_IDX, wallet.clone());
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&wallet_key)
            .unwrap_or(Vec::new(&env));
        ids.push_back(new_id);
        env.storage().persistent().set(&wallet_key, &ids);

        // Extend storage TTL (approx 1 year in ledgers)
        env.storage().persistent().extend_ttl(&sub_key, 0, 3_110_400);
        env.storage().persistent().extend_ttl(&wallet_key, 0, 3_110_400);

        emit_subscription_created(&env, new_id, &wallet);
        log!(&env, "Subscription created: id={}", new_id);

        new_id
    }

    /// Update mutable fields of an existing subscription.
    pub fn update_subscription(
        env: Env,
        caller: Address,
        id: u64,
        merchant_name: Option<String>,
        frequency: Option<Frequency>,
        amount: Option<i128>,
        next_payment_ledger: Option<u64>,
        custom_label: Option<String>,
    ) {
        caller.require_auth();

        let sub_key = (SUB_KEY, id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&sub_key)
            .expect("Subscription not found");

        // Ensure only the wallet owner can update
        sub.wallet.require_auth();

        if let Some(name) = merchant_name {
            sub.merchant_name = name;
        }
        if let Some(freq) = frequency {
            sub.frequency = freq;
        }
        if let Some(amt) = amount {
            sub.amount = amt;
        }
        if let Some(next) = next_payment_ledger {
            sub.next_payment_ledger = next;
        }
        if let Some(label) = custom_label {
            sub.custom_label = Some(label);
        }

        env.storage().persistent().set(&sub_key, &sub);
        env.storage().persistent().extend_ttl(&sub_key, 0, 3_110_400);

        emit_subscription_updated(&env, id, &sub.wallet);
    }

    /// Mark a subscription as cancelled.
    pub fn cancel_subscription(env: Env, caller: Address, id: u64) {
        caller.require_auth();

        let sub_key = (SUB_KEY, id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&sub_key)
            .expect("Subscription not found");

        sub.wallet.require_auth();
        sub.status = SubscriptionStatus::Cancelled;

        env.storage().persistent().set(&sub_key, &sub);
        emit_subscription_cancelled(&env, id, &sub.wallet);
    }

    /// Pause a subscription temporarily.
    pub fn pause_subscription(env: Env, caller: Address, id: u64) {
        caller.require_auth();

        let sub_key = (SUB_KEY, id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&sub_key)
            .expect("Subscription not found");

        sub.wallet.require_auth();
        sub.status = SubscriptionStatus::Paused;

        env.storage().persistent().set(&sub_key, &sub);
    }

    /// Resume a paused subscription.
    pub fn resume_subscription(env: Env, caller: Address, id: u64) {
        caller.require_auth();

        let sub_key = (SUB_KEY, id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&sub_key)
            .expect("Subscription not found");

        sub.wallet.require_auth();
        sub.status = SubscriptionStatus::Active;

        env.storage().persistent().set(&sub_key, &sub);
    }

    /// Called by the backend after confirming a payment occurred — records
    /// the last payment ledger and advances next_payment_ledger.
    pub fn confirm_payment(
        env: Env,
        caller: Address,
        id: u64,
        payment_ledger: u64,
        next_payment_ledger: u64,
    ) {
        caller.require_auth();

        let sub_key = (SUB_KEY, id);
        let mut sub: Subscription = env
            .storage()
            .persistent()
            .get(&sub_key)
            .expect("Subscription not found");

        sub.last_payment_ledger = payment_ledger;
        sub.next_payment_ledger = next_payment_ledger;

        env.storage().persistent().set(&sub_key, &sub);
        env.storage().persistent().extend_ttl(&sub_key, 0, 3_110_400);
    }

    // ── Read Functions ────────────────────────────────────────────────────────

    /// Get a single subscription by its ID.
    pub fn get_subscription(env: Env, id: u64) -> Option<Subscription> {
        let sub_key = (SUB_KEY, id);
        env.storage().persistent().get(&sub_key)
    }

    /// List all subscription IDs registered for a wallet address.
    pub fn list_wallet_subscription_ids(env: Env, wallet: Address) -> Vec<u64> {
        let wallet_key = (WALLET_IDX, wallet);
        env.storage()
            .persistent()
            .get(&wallet_key)
            .unwrap_or(Vec::new(&env))
    }

    /// List all active subscriptions for a wallet.
    pub fn list_active_subscriptions(env: Env, wallet: Address) -> Vec<Subscription> {
        let wallet_key = (WALLET_IDX, wallet);
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&wallet_key)
            .unwrap_or(Vec::new(&env));

        let mut active = Vec::new(&env);
        for id in ids.iter() {
            let sub_key = (SUB_KEY, id);
            if let Some(sub) = env.storage().persistent().get::<_, Subscription>(&sub_key) {
                if sub.status == SubscriptionStatus::Active {
                    active.push_back(sub);
                }
            }
        }
        active
    }

    /// Count subscriptions for a wallet (all statuses).
    pub fn count_wallet_subscriptions(env: Env, wallet: Address) -> u32 {
        let wallet_key = (WALLET_IDX, wallet);
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&wallet_key)
            .unwrap_or(Vec::new(&env));
        ids.len()
    }

    /// Get global subscription counter (total ever created).
    pub fn total_subscriptions(env: Env) -> u64 {
        env.storage().instance().get(&SUB_COUNT).unwrap_or(0u64)
    }
}

mod test;