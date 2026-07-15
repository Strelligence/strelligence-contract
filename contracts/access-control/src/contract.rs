use soroban_sdk::{contract, contractimpl, Address, Bytes, Env};

use crate::{
    errors::ContractError,
    events,
    storage::{DataKey, INITIAL_VERSION, STORAGE_TTL},
    types::{Role, RoleAssignment},
};

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    // ─────────────────────────────────────────────────────────────────────────
    // INITIALIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Initialize the access control contract with an admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        let placeholder = Bytes::from_array(&env, &[0u8; 32]);
        env.storage().instance().set(&DataKey::Admin, &admin);

        let admin_assignment = RoleAssignment {
            address: admin.clone(),
            role: Role::Admin,
            granted_at_ledger: env.ledger().sequence() as u64,
            expires_at_ledger: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::RoleAssignment(admin.clone()), &admin_assignment);

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
    // ROLE MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Grant a role to an address. Only admin can call this.
    pub fn grant_role(
        env: Env,
        caller: Address,
        grantee: Address,
        role: Role,
        expires_at_ledger: Option<u64>,
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

        if env
            .storage()
            .instance()
            .has(&DataKey::RoleAssignment(grantee.clone()))
        {
            return Err(ContractError::RoleAlreadyAssigned);
        }

        let role_num = match role {
            Role::Admin => 0,
            Role::Operator => 1,
            Role::Viewer => 2,
        };

        let assignment = RoleAssignment {
            address: grantee.clone(),
            role,
            granted_at_ledger: env.ledger().sequence() as u64,
            expires_at_ledger,
        };

        env.storage()
            .instance()
            .set(&DataKey::RoleAssignment(grantee.clone()), &assignment);

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::role_granted(&env, &admin, &grantee, role_num);

        Ok(())
    }

    /// Revoke a role from an address. Only admin can call this.
    pub fn revoke_role(env: Env, caller: Address, grantee: Address) -> Result<(), ContractError> {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            return Err(ContractError::NotAdmin);
        }

        if !env
            .storage()
            .instance()
            .has(&DataKey::RoleAssignment(grantee.clone()))
        {
            return Err(ContractError::RoleNotFound);
        }

        env.storage()
            .instance()
            .remove(&DataKey::RoleAssignment(grantee.clone()));

        env.storage().instance().extend_ttl(0, 6_312_000);

        events::role_revoked(&env, &admin, &grantee);

        Ok(())
    }

    /// Check if an address has a specific role.
    pub fn has_role(env: Env, address: Address, role: Role) -> bool {
        if let Some(assignment) = env
            .storage()
            .instance()
            .get::<_, RoleAssignment>(&DataKey::RoleAssignment(address))
        {
            if assignment.role == role {
                if let Some(expires) = assignment.expires_at_ledger {
                    let current_ledger = env.ledger().sequence() as u64;
                    return current_ledger < expires;
                }
                return true;
            }
        }
        false
    }

    /// Get role assignment for an address.
    pub fn get_role(env: Env, address: Address) -> Option<RoleAssignment> {
        env.storage()
            .instance()
            .get(&DataKey::RoleAssignment(address))
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
