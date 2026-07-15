#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    use crate::contract::CrossContractRouter;
    use crate::contract::CrossContractRouterClient;
    use crate::errors::ContractError;
    use crate::types::CallStatus;

    fn setup<'a>() -> (Env, CrossContractRouterClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrossContractRouter);
        let client = CrossContractRouterClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (
        Env,
        CrossContractRouterClient<'a>,
        Address,
        Address,
        Address,
        Address,
    ) {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let recurring = Address::generate(&env);
        let metadata = Address::generate(&env);
        let automation = Address::generate(&env);

        client.initialize(&admin, &recurring, &metadata, &automation);

        (env, client, admin, recurring, metadata, automation)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // initialization
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let (env, client, admin, recurring, metadata, automation) = setup_initialized();

        assert_eq!(client.get_admin(), Some(admin));
        let registry = client.get_registry().unwrap();
        assert_eq!(registry.recurring_registry, recurring);
        assert_eq!(registry.metadata_registry, metadata);
        assert_eq!(registry.automation_rules, automation);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client, admin, recurring, metadata, automation) = setup_initialized();

        let result = client.try_initialize(&admin, &recurring, &metadata, &automation);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // cross-contract operations
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_sub_with_meta_success() {
        let (env, client, admin, _, _, _) = setup_initialized();

        let owner = Address::generate(&env);
        let mut tags = Vec::new(&env);
        tags.push_back(String::from_str(&env, "recurring"));

        let call_id = client.create_sub_with_meta(
            &admin,
            &owner,
            &String::from_str(&env, "Netflix"),
            &1,
            &10_000_000,
            &1,
            &tags,
            &Some(String::from_str(&env, "Subscription")),
        );

        assert_eq!(call_id, 1);
        assert_eq!(client.total_calls(), 1);
        assert!(client.is_processed(&call_id));

        let call = client.get_cross_call(&call_id).unwrap();
        assert_eq!(call.status, CallStatus::Success);
    }

    #[test]
    fn test_batch_update_metadata_success() {
        let (env, client, admin, _, _, _) = setup_initialized();

        let mut tx_hashes = Vec::new(&env);
        tx_hashes.push_back(String::from_str(&env, "tx1"));
        tx_hashes.push_back(String::from_str(&env, "tx2"));

        let call_id = client.batch_update_metadata(&admin, &tx_hashes, &Some(1));

        assert_eq!(call_id, 1);
        assert!(client.is_processed(&call_id));
    }

    #[test]
    fn test_execute_rule_with_subscription_success() {
        let (env, client, admin, _, _, _) = setup_initialized();

        let call_id = client.execute_rule_with_subscription(&admin, &1, &1);

        assert_eq!(call_id, 1);
        assert!(client.is_processed(&call_id));
    }

    #[test]
    fn test_cross_call_unauthorized_fails() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrossContractRouter);
        let client = CrossContractRouterClient::new(&env, &contract_id);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let recurring = Address::generate(&env);
        let metadata = Address::generate(&env);
        let automation = Address::generate(&env);

        client.initialize(&admin, &recurring, &metadata, &automation);

        let non_admin = Address::generate(&env);
        let owner = Address::generate(&env);

        let result = client.try_create_sub_with_meta(
            &non_admin,
            &owner,
            &String::from_str(&env, "Netflix"),
            &1,
            &10_000_000,
            &1,
            &Vec::new(&env),
            &None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::Unauthorized);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // read functions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_cross_call_not_found() {
        let (env, client) = setup();
        assert!(client.get_cross_call(&999).is_none());
    }

    #[test]
    fn test_is_processed_returns_false_for_unknown() {
        let (env, client) = setup();
        assert!(!client.is_processed(&999));
    }

    #[test]
    fn test_total_calls() {
        let (env, client, admin, _, _, _) = setup_initialized();

        assert_eq!(client.total_calls(), 0);

        client.create_sub_with_meta(
            &admin,
            &Address::generate(&env),
            &String::from_str(&env, "Netflix"),
            &1,
            &10_000_000,
            &1,
            &Vec::new(&env),
            &None,
        );

        assert_eq!(client.total_calls(), 1);
    }
}
