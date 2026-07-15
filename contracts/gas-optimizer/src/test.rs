#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    use crate::contract::GasOptimizerContract;
    use crate::contract::GasOptimizerContractClient;
    use crate::errors::ContractError;
    use crate::types::StorageOptimization;

    fn setup<'a>() -> (Env, GasOptimizerContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, GasOptimizerContract);
        let client = GasOptimizerContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, GasOptimizerContractClient<'a>, Address) {
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

        let settings = client.get_settings().unwrap();
        assert_eq!(settings.batch_size, 100);
        assert!(settings.compress);
        assert!(settings.cache_enabled);
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
    // gas profiling
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_record_gas_profile_success() {
        let (env, client, admin) = setup_initialized();

        let id = client.record_gas_profile(&admin, &1, &10, &5, &1024, &512);

        assert_eq!(id, 1);
        assert_eq!(client.total_profiles(), 1);

        let profile = client.get_gas_profile(&id).unwrap();
        assert_eq!(profile.operation, 1);
        assert_eq!(profile.read_count, 10);
        assert_eq!(profile.write_count, 5);
    }

    #[test]
    fn test_record_gas_profile_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);

        let result = client.try_record_gas_profile(&non_admin, &1, &10, &5, &1024, &512);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::Unauthorized);
    }

    #[test]
    fn test_record_multiple_profiles() {
        let (env, client, admin) = setup_initialized();

        client.record_gas_profile(&admin, &1, &10, &5, &1024, &512);
        client.record_gas_profile(&admin, &2, &20, &10, &2048, &1024);

        assert_eq!(client.total_profiles(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // optimization settings
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_update_settings_success() {
        let (env, client, admin) = setup_initialized();

        client.update_settings(&admin, &50, &false, &true);

        let settings = client.get_settings().unwrap();
        assert_eq!(settings.batch_size, 50);
        assert!(!settings.compress);
        assert!(settings.cache_enabled);
    }

    #[test]
    fn test_update_settings_invalid_batch_size_zero() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_update_settings(&admin, &0, &true, &true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidBatchSize
        );
    }

    #[test]
    fn test_update_settings_invalid_batch_size_too_large() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_update_settings(&admin, &1001, &true, &true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidBatchSize
        );
    }

    #[test]
    fn test_update_settings_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);

        let result = client.try_update_settings(&non_admin, &50, &true, &true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // read functions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_gas_profile_not_found() {
        let (env, client) = setup();
        assert!(client.get_gas_profile(&999).is_none());
    }

    #[test]
    fn test_total_profiles() {
        let (env, client, admin) = setup_initialized();

        assert_eq!(client.total_profiles(), 0);

        client.record_gas_profile(&admin, &1, &10, &5, &1024, &512);

        assert_eq!(client.total_profiles(), 1);
    }
}
