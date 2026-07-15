#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, Symbol};

    use crate::contract::EventFilterContract;
    use crate::contract::EventFilterContractClient;
    use crate::errors::ContractError;
    use crate::types::EventFilter;

    fn setup<'a>() -> (Env, EventFilterContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, EventFilterContract);
        let client = EventFilterContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, EventFilterContractClient<'a>, Address) {
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
    // event indexing
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_record_event_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "test_topic"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        assert_eq!(id, 1);
        assert_eq!(client.total_events(), 1);

        let event = client.get_event(&id).unwrap();
        assert_eq!(event.topic, Symbol::new(&env, "test_topic"));
    }

    #[test]
    fn test_record_event_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);
        let contract_addr = Address::generate(&env);

        let result = client.try_record_event(
            &non_admin,
            &contract_addr,
            &Symbol::new(&env, "test"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_record_multiple_events() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic2"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        assert_eq!(client.total_events(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // event querying
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_query_events_no_filter() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        let filter = EventFilter {
            topic: None,
            from_ledger: None,
            to_ledger: None,
            contract_address: None,
        };

        let result = client.query_events(&filter, &None, &10);
        assert_eq!(result.events.len(), 1);
        assert!(result.next_cursor.is_none());
    }

    #[test]
    fn test_query_events_with_topic_filter() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic2"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        let filter = EventFilter {
            topic: Some(Symbol::new(&env, "topic1")),
            from_ledger: None,
            to_ledger: None,
            contract_address: None,
        };

        let result = client.query_events(&filter, &None, &10);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn test_get_events_by_topic() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic1"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        let results = client.get_events_by_topic(&Symbol::new(&env, "topic1"), &10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_event_not_found() {
        let (env, client) = setup();
        assert!(client.get_event(&999).is_none());
    }

    #[test]
    fn test_total_events() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        assert_eq!(client.total_events(), 0);

        client.record_event(
            &admin,
            &contract_addr,
            &Symbol::new(&env, "topic"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(client.total_events(), 1);
    }
}
