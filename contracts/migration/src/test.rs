#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    use crate::contract::MigrationContract;
    use crate::contract::MigrationContractClient;
    use crate::errors::ContractError;
    use crate::types::MigrationStatus;

    fn setup<'a>() -> (Env, MigrationContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, MigrationContract);
        let client = MigrationContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, MigrationContractClient<'a>, Address) {
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
    // migration management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_plan_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        assert_eq!(id, 1);
        assert_eq!(client.total_plans(), 1);

        let plan = client.get_plan(&id).unwrap();
        assert_eq!(plan.from_version, 1);
        assert_eq!(plan.to_version, 2);
        assert_eq!(plan.status, MigrationStatus::Pending);
    }

    #[test]
    fn test_create_plan_same_hash_fails() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let result = client.try_create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[1u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::SameWasmHash);
    }

    #[test]
    fn test_create_plan_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);
        let contract_addr = Address::generate(&env);

        let result = client.try_create_plan(
            &non_admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_start_migration_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        client.start_migration(&admin, &id);

        let plan = client.get_plan(&id).unwrap();
        assert_eq!(plan.status, MigrationStatus::InProgress);
    }

    #[test]
    fn test_complete_migration_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        client.start_migration(&admin, &id);
        client.complete_migration(&admin, &id);

        let plan = client.get_plan(&id).unwrap();
        assert_eq!(plan.status, MigrationStatus::Completed);
    }

    #[test]
    fn test_rollback_migration_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        client.start_migration(&admin, &id);
        client.rollback_migration(&admin, &id);

        let plan = client.get_plan(&id).unwrap();
        assert_eq!(plan.status, MigrationStatus::RolledBack);
    }

    #[test]
    fn test_verify_plan_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        client.start_migration(&admin, &id);
        client.complete_migration(&admin, &id);

        let verification = client.verify_plan(&admin, &id);
        assert!(verification.storage_valid);
        assert!(verification.state_valid);
        assert!(verification.events_valid);
    }

    #[test]
    fn test_get_plan_not_found() {
        let (env, client) = setup();
        assert!(client.get_plan(&999).is_none());
    }

    #[test]
    fn test_total_plans() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        assert_eq!(client.total_plans(), 0);

        client.create_plan(
            &admin,
            &contract_addr,
            &1,
            &2,
            &Bytes::from_array(&env, &[1u8; 32]),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        assert_eq!(client.total_plans(), 1);
    }
}
