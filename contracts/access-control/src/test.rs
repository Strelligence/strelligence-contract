#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    use crate::contract::AccessControlContract;
    use crate::contract::AccessControlContractClient;
    use crate::errors::ContractError;
    use crate::types::Role;

    fn setup<'a>() -> (Env, AccessControlContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, AccessControlContract);
        let client = AccessControlContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, AccessControlContractClient<'a>, Address) {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // initialization
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let (env, client, admin) = setup_initialized();

        assert_eq!(client.get_admin(), Some(admin.clone()));
        assert_eq!(client.get_version(), 1);
        assert!(client.has_role(&admin, &Role::Admin));
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_initialize(&admin);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // role management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_grant_role_success() {
        let (env, client, admin) = setup_initialized();
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &None);

        assert!(client.has_role(&operator, &Role::Operator));
        assert!(!client.has_role(&operator, &Role::Admin));
    }

    #[test]
    fn test_grant_role_viewer() {
        let (env, client, admin) = setup_initialized();
        let viewer = Address::generate(&env);

        client.grant_role(&admin, &viewer, &Role::Viewer, &None);

        assert!(client.has_role(&viewer, &Role::Viewer));
    }

    #[test]
    fn test_grant_role_temporary() {
        let (env, client, admin) = setup_initialized();
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &Some(1000));

        assert!(client.has_role(&operator, &Role::Operator));
    }

    #[test]
    fn test_grant_role_already_assigned_fails() {
        let (env, client, admin) = setup_initialized();
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &None);

        let result = client.try_grant_role(&admin, &operator, &Role::Viewer, &None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::RoleAlreadyAssigned
        );
    }

    #[test]
    fn test_grant_role_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);
        let operator = Address::generate(&env);

        let result = client.try_grant_role(&non_admin, &operator, &Role::Operator, &None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_revoke_role_success() {
        let (env, client, admin) = setup_initialized();
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &None);
        assert!(client.has_role(&operator, &Role::Operator));

        client.revoke_role(&admin, &operator);
        assert!(!client.has_role(&operator, &Role::Operator));
    }

    #[test]
    fn test_revoke_role_not_found_fails() {
        let (env, client, admin) = setup_initialized();
        let unknown = Address::generate(&env);

        let result = client.try_revoke_role(&admin, &unknown);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::RoleNotFound);
    }

    #[test]
    fn test_revoke_role_unauthorized_fails() {
        let (env, client, admin) = setup_initialized();
        let non_admin = Address::generate(&env);
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &None);

        let result = client.try_revoke_role(&non_admin, &operator);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_has_role_returns_false_for_unknown() {
        let (env, client) = setup();
        let unknown = Address::generate(&env);

        assert!(!client.has_role(&unknown, &Role::Admin));
    }

    #[test]
    fn test_get_role_returns_assignment() {
        let (env, client, admin) = setup_initialized();
        let operator = Address::generate(&env);

        client.grant_role(&admin, &operator, &Role::Operator, &None);

        let assignment = client.get_role(&operator).unwrap();
        assert_eq!(assignment.address, operator);
    }

    #[test]
    fn test_get_role_returns_none_for_unknown() {
        let (env, client) = setup();
        let unknown = Address::generate(&env);

        assert!(client.get_role(&unknown).is_none());
    }
}
