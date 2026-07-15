#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String};

    use crate::contract::EmergencyPauseContract;
    use crate::contract::EmergencyPauseContractClient;
    use crate::errors::ContractError;

    fn setup<'a>() -> (Env, EmergencyPauseContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, EmergencyPauseContract);
        let client = EmergencyPauseContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, EmergencyPauseContractClient<'a>, Address) {
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
        assert!(!client.is_paused());
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
    // pause management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_pause_success() {
        let (env, client, admin) = setup_initialized();

        client.pause(&admin, &String::from_str(&env, "security incident"), &None);

        assert!(client.is_paused());

        let state = client.get_pause_state();
        assert!(state.paused);
        assert_eq!(state.reason, String::from_str(&env, "security incident"));
    }

    #[test]
    fn test_pause_with_auto_unpause() {
        let (env, client, admin) = setup_initialized();

        client.pause(&admin, &String::from_str(&env, "maintenance"), &Some(1000));

        let state = client.get_pause_state();
        assert_eq!(state.unpause_at_ledger, Some(1000));
    }

    #[test]
    fn test_pause_already_paused_fails() {
        let (env, client, admin) = setup_initialized();

        client.pause(&admin, &String::from_str(&env, "first"), &None);

        let result = client.try_pause(&admin, &String::from_str(&env, "second"), &None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyPaused);
    }

    #[test]
    fn test_pause_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);

        let result = client.try_pause(&non_admin, &String::from_str(&env, "reason"), &None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_unpause_success() {
        let (env, client, admin) = setup_initialized();

        client.pause(&admin, &String::from_str(&env, "reason"), &None);
        assert!(client.is_paused());

        client.unpause(&admin);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_unpause_not_paused_fails() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_unpause(&admin);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotPaused);
    }

    #[test]
    fn test_unpause_unauthorized_fails() {
        let (env, client, admin) = setup_initialized();
        let non_admin = Address::generate(&env);

        client.pause(&admin, &String::from_str(&env, "reason"), &None);

        let result = client.try_unpause(&non_admin);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_pause_history() {
        let (env, client, admin) = setup_initialized();

        let history = client.get_pause_history();
        assert_eq!(history.pause_count, 0);

        client.pause(&admin, &String::from_str(&env, "reason"), &None);
        client.unpause(&admin);

        let history = client.get_pause_history();
        assert_eq!(history.pause_count, 1);
    }
}
