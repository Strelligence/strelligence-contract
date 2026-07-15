#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    use crate::contract::VersioningContract;
    use crate::contract::VersioningContractClient;
    use crate::errors::ContractError;

    fn setup<'a>() -> (Env, VersioningContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, VersioningContract);
        let client = VersioningContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, VersioningContractClient<'a>, Address) {
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

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_version(), 1);
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
    // version management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_register_version_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.register_version(
            &admin,
            &contract_addr,
            &1,
            &0,
            &0,
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        assert_eq!(id, 1);
        assert_eq!(client.total_records(), 1);

        let record = client.get_version_record(&id).unwrap();
        assert_eq!(record.version.major, 1);
        assert_eq!(record.version.minor, 0);
        assert_eq!(record.version.patch, 0);
        assert!(!record.deprecated);
    }

    #[test]
    fn test_register_version_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);
        let contract_addr = Address::generate(&env);

        let result = client.try_register_version(
            &non_admin,
            &contract_addr,
            &1,
            &0,
            &0,
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_deprecate_version_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.register_version(
            &admin,
            &contract_addr,
            &1,
            &0,
            &0,
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        client.deprecate_version(&admin, &id);

        let record = client.get_version_record(&id).unwrap();
        assert!(record.deprecated);
    }

    #[test]
    fn test_deprecate_version_not_found_fails() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_deprecate_version(&admin, &999);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::VersionNotFound);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // compatibility checks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_check_compatibility_same_version() {
        let (env, client) = setup();

        let result = client.check_compatibility(&1, &0, &0, &1, &0, &0);
        assert!(result.compatible);
        assert!(!result.breaking_changes);
    }

    #[test]
    fn test_check_compatibility_minor_update() {
        let (env, client) = setup();

        let result = client.check_compatibility(&1, &0, &0, &1, &1, &0);
        assert!(result.compatible);
        assert!(!result.breaking_changes);
    }

    #[test]
    fn test_check_compatibility_major_update() {
        let (env, client) = setup();

        let result = client.check_compatibility(&1, &0, &0, &2, &0, &0);
        assert!(!result.compatible);
        assert!(result.breaking_changes);
    }

    #[test]
    fn test_get_version_record_not_found() {
        let (env, client) = setup();
        assert!(client.get_version_record(&999).is_none());
    }

    #[test]
    fn test_total_records() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        assert_eq!(client.total_records(), 0);

        client.register_version(
            &admin,
            &contract_addr,
            &1,
            &0,
            &0,
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(client.total_records(), 1);
    }
}
