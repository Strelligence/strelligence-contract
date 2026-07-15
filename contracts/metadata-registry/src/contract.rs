use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{Metadata, TransactionCategory, TransactionSentiment},
};

#[contract]
pub struct MetadataRegistryContract;

#[contractimpl]
impl MetadataRegistryContract {
    // ─────────────────────────────────────────────────────────────────────────
    // WRITE FUNCTIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Attach metadata to a Stellar transaction hash.
    ///
    /// Called by the backend after AI classification, or manually by the
    /// wallet owner. The tx_hash must not already have metadata.
    pub fn add_metadata(
        env: Env,
        caller: Address,
        tx_hash: String,
        category: TransactionCategory,
        sentiment: TransactionSentiment,
        tags: Vec<String>,
        label: Option<String>,
        notes: Option<String>,
        counterparty_label: Option<String>,
        is_recurring: bool,
        recurring_id: Option<u64>,
        ai_confidence: u32,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        if ai_confidence > 100 {
            return Err(ContractError::InvalidConfidence);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Metadata(tx_hash.clone()))
        {
            return Err(ContractError::AlreadyExists);
        }

        let owner = caller.clone();
        let ledger = env.ledger().sequence() as u64;

        let metadata = Metadata {
            tx_hash: tx_hash.clone(),
            owner: owner.clone(),
            category,
            sentiment,
            tags,
            label,
            notes,
            counterparty_label,
            is_recurring,
            recurring_id,
            ai_confidence,
            created_at_ledger: ledger,
            updated_at_ledger: ledger,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Metadata(tx_hash.clone()), &metadata);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Metadata(tx_hash.clone()), 0, STORAGE_TTL);

        let mut tx_hashes: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletTxHashes(owner.clone()))
            .unwrap_or(Vec::new(&env));
        tx_hashes.push_back(tx_hash.clone());
        env.storage()
            .persistent()
            .set(&DataKey::WalletTxHashes(owner.clone()), &tx_hashes);
        env.storage().persistent().extend_ttl(
            &DataKey::WalletTxHashes(owner.clone()),
            0,
            STORAGE_TTL,
        );

        events::metadata_added(&env, &owner, &tx_hash);

        Ok(())
    }

    /// Update mutable fields of an existing metadata record.
    /// Only the metadata owner can call this.
    pub fn update_metadata(
        env: Env,
        caller: Address,
        tx_hash: String,
        category: Option<TransactionCategory>,
        sentiment: Option<TransactionSentiment>,
        tags: Option<Vec<String>>,
        label: Option<String>,
        notes: Option<String>,
        counterparty_label: Option<String>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut meta: Metadata = env
            .storage()
            .persistent()
            .get(&DataKey::Metadata(tx_hash.clone()))
            .ok_or(ContractError::MetadataNotFound)?;

        meta.owner.require_auth();

        if let Some(c) = category {
            meta.category = c;
        }
        if let Some(s) = sentiment {
            meta.sentiment = s;
        }
        if let Some(t) = tags {
            meta.tags = t;
        }
        if label.is_some() {
            meta.label = label;
        }
        if notes.is_some() {
            meta.notes = notes;
        }
        if counterparty_label.is_some() {
            meta.counterparty_label = counterparty_label;
        }

        meta.updated_at_ledger = env.ledger().sequence() as u64;

        env.storage()
            .persistent()
            .set(&DataKey::Metadata(tx_hash.clone()), &meta);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Metadata(tx_hash.clone()), 0, STORAGE_TTL);

        events::metadata_updated(&env, &meta.owner, &tx_hash);

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

    /// Fetch metadata for a single transaction hash.
    pub fn get_metadata(env: Env, tx_hash: String) -> Option<Metadata> {
        env.storage().persistent().get(&DataKey::Metadata(tx_hash))
    }

    /// List all transaction hashes that have metadata for a wallet.
    pub fn get_wallet_metadata(env: Env, owner: Address) -> Vec<String> {
        env.storage()
            .persistent()
            .get(&DataKey::WalletTxHashes(owner))
            .unwrap_or(Vec::new(&env))
    }

    /// Filter metadata records by category for a given wallet.
    pub fn get_metadata_by_category(
        env: Env,
        owner: Address,
        category: TransactionCategory,
    ) -> Vec<Metadata> {
        let tx_hashes: Vec<String> = env
            .storage()
            .persistent()
            .get(&DataKey::WalletTxHashes(owner))
            .unwrap_or(Vec::new(&env));

        let mut results = Vec::new(&env);
        for tx_hash in tx_hashes.iter() {
            if let Some(meta) = env
                .storage()
                .persistent()
                .get::<_, Metadata>(&DataKey::Metadata(tx_hash))
            {
                if meta.category == category {
                    results.push_back(meta);
                }
            }
        }
        results
    }
}
