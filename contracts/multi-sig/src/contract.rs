use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String, Vec};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{Proposal, ProposalStatus, Signer},
};

#[contract]
pub struct MultiSigContract;

#[contractimpl]
impl MultiSigContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the multi-sig contract with an admin, initial signers, and threshold.
    pub fn initialize(
        env: Env,
        admin: Address,
        signers: Vec<Address>,
        threshold: u32,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let signer_count = signers.len();
        if threshold == 0 || threshold > signer_count {
            return Err(ContractError::InvalidThreshold);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &0u64);

        let mut signer_list: Vec<Signer> = Vec::new(&env);
        let ledger = env.ledger().sequence() as u64;

        for signer_addr in signers.iter() {
            let signer = Signer {
                address: signer_addr.clone(),
                added_at_ledger: ledger,
                active: true,
            };
            signer_list.push_back(signer);
            env.storage().persistent().set(
                &DataKey::SignerProposalVote(signer_addr.clone(), 0u64),
                &false,
            );
        }

        env.storage()
            .instance()
            .set(&DataKey::Signers, &signer_list);
        env.storage().instance().extend_ttl(0, 6_312_000);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SIGNER MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Add a new signer. Only admin can call this.
    pub fn add_signer(env: Env, caller: Address, signer: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        let mut signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        for s in signers.iter() {
            if s.address == signer && s.active {
                return Err(ContractError::SignerAlreadyExists);
            }
        }

        let ledger = env.ledger().sequence() as u64;
        let new_signer = Signer {
            address: signer.clone(),
            added_at_ledger: ledger,
            active: true,
        };
        signers.push_back(new_signer);

        env.storage()
            .instance()
            .set(&DataKey::Signers, &signers);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::signer_added(&env, &admin, &signer);

        Ok(())
    }

    /// Remove a signer. Only admin can call this. Cannot remove yourself.
    pub fn remove_signer(env: Env, caller: Address, signer: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        if caller == signer {
            return Err(ContractError::CannotRemoveSelf);
        }

        let mut signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        let mut found = false;
        let mut result_signers: Vec<Signer> = Vec::new(&env);

        for s in signers.iter() {
            if s.address == signer && s.active {
                let deactivated = Signer {
                    address: s.address.clone(),
                    added_at_ledger: s.added_at_ledger,
                    active: false,
                };
                result_signers.push_back(deactivated);
                found = true;
            } else {
                result_signers.push_back(s);
            }
        }

        if !found {
            return Err(ContractError::SignerNotFound);
        }

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or(0);
        let active_count = result_signers.iter().filter(|s| s.active).count() as u32;

        if threshold > active_count {
            return Err(ContractError::ThresholdExceedsSigners);
        }

        env.storage()
            .instance()
            .set(&DataKey::Signers, &result_signers);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::signer_removed(&env, &admin, &signer);

        Ok(())
    }

    /// Update the signature threshold. Only admin can call this.
    pub fn update_threshold(
        env: Env,
        caller: Address,
        new_threshold: u32,
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

        if new_threshold == 0 {
            return Err(ContractError::InvalidThreshold);
        }

        let signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        let active_count = signers.iter().filter(|s| s.active).count() as u32;

        if new_threshold > active_count {
            return Err(ContractError::ThresholdExceedsSigners);
        }

        let old_threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::Threshold, &new_threshold);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::threshold_updated(&env, &admin, old_threshold, new_threshold);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PROPOSAL MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Create a new proposal. Only signers can call this.
    pub fn create_proposal(
        env: Env,
        caller: Address,
        target_contract: Address,
        function_name: String,
        payload: Bytes,
    ) -> Result<u64, ContractError> {
        caller.require_auth();

        let signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        let is_signer = signers.iter().any(|s| s.address == caller && s.active);
        if !is_signer {
            return Err(ContractError::Unauthorized);
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(0u64)
            + 1;

        let proposal = Proposal {
            id,
            proposer: caller.clone(),
            target_contract,
            function_name,
            payload,
            created_at_ledger: env.ledger().sequence() as u64,
            executed: false,
            cancelled: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::NextProposalId, &id);

        let signers_voted: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::ProposalSigners(id), &signers_voted);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::proposal_created(&env, &caller, id);

        Ok(id)
    }

    /// Sign a proposal. Only signers can call this.
    pub fn sign_proposal(
        env: Env,
        caller: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        let is_signer = signers.iter().any(|s| s.address == caller && s.active);
        if !is_signer {
            return Err(ContractError::Unauthorized);
        }

        let proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ContractError::ProposalNotFound)?;

        if proposal.executed {
            return Err(ContractError::AlreadyExecuted);
        }
        if proposal.cancelled {
            return Err(ContractError::ProposalNotFound);
        }

        let mut signed_by: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ProposalSigners(proposal_id))
            .unwrap_or(Vec::new(&env));

        if signed_by.iter().any(|s| s == caller) {
            return Err(ContractError::AlreadySigned);
        }

        signed_by.push_back(caller.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ProposalSigners(proposal_id), &signed_by);

        events::proposal_signed(&env, &caller, proposal_id);

        Ok(())
    }

    /// Execute a proposal once threshold is met. Only signers can call this.
    pub fn execute_proposal(
        env: Env,
        caller: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let signers: Vec<Signer> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .ok_or(ContractError::NotInitialized)?;

        let is_signer = signers.iter().any(|s| s.address == caller && s.active);
        if !is_signer {
            return Err(ContractError::Unauthorized);
        }

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ContractError::ProposalNotFound)?;

        if proposal.executed {
            return Err(ContractError::AlreadyExecuted);
        }
        if proposal.cancelled {
            return Err(ContractError::ProposalNotFound);
        }

        let signed_by: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ProposalSigners(proposal_id))
            .unwrap_or(Vec::new(&env));

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or(0);

        if (signed_by.len() as u32) < threshold {
            return Err(ContractError::NotReadyToExecute);
        }

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::proposal_executed(&env, proposal_id);

        Ok(())
    }

    /// Cancel a proposal. Only the proposer or admin can call this.
    pub fn cancel_proposal(
        env: Env,
        caller: Address,
        proposal_id: u64,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(ContractError::ProposalNotFound)?;

        if proposal.executed {
            return Err(ContractError::AlreadyExecuted);
        }
        if proposal.cancelled {
            return Err(ContractError::ProposalNotFound);
        }

        if caller != proposal.proposer && caller != admin {
            return Err(ContractError::Unauthorized);
        }

        proposal.cancelled = true;
        env.storage()
            .instance()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().extend_ttl(0, 6_312_000);

        events::proposal_cancelled(&env, proposal_id);

        Ok(())
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

    /// Get the current threshold.
    pub fn get_threshold(env: Env) -> Option<u32> {
        env.storage().instance().get(&DataKey::Threshold)
    }

    /// Get all signers.
    pub fn get_signers(env: Env) -> Vec<Signer> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or(Vec::new(&env))
    }

    /// Get a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<Proposal> {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
    }

    /// Get the list of signers who have signed a proposal.
    pub fn get_proposal_signers(env: Env, proposal_id: u64) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::ProposalSigners(proposal_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Check if a signer has signed a proposal.
    pub fn has_signed(env: Env, signer: Address, proposal_id: u64) -> bool {
        let signed_by: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ProposalSigners(proposal_id))
            .unwrap_or(Vec::new(&env));

        signed_by.iter().any(|s| s == signer)
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

    /// Get the total number of proposals.
    pub fn total_proposals(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NextProposalId)
            .unwrap_or(0u64)
    }
}
