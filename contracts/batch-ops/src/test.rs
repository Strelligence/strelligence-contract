#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, Vec};

    use crate::contract::BatchOpsContract;
    use crate::contract::BatchOpsContractClient;
    use crate::errors::ContractError;
    use crate::types::{BatchOpStatus, BatchOpType, BatchOperation, BatchStatus};

    fn setup<'a>() -> (Env, BatchOpsContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, BatchOpsContract);
        let client = BatchOpsContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, BatchOpsContractClient<'a>, Address) {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    fn create_operation(env: &Env, op_type: BatchOpType) -> BatchOperation {
        BatchOperation {
            id: 1,
            operation_type: op_type,
            target: Address::generate(env),
            payload: Bytes::from_array(env, &[0u8; 32]),
            status: BatchOpStatus::Pending,
        }
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
    // batch operations
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_execute_batch_success() {
        let (env, client, admin) = setup_initialized();

        let mut ops = Vec::new(&env);
        ops.push_back(create_operation(&env, BatchOpType::Create));
        ops.push_back(create_operation(&env, BatchOpType::Update));

        let result = client.execute_batch(&admin, &ops);

        assert_eq!(result.batch_id, 1);
        assert_eq!(result.total_operations, 2);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 0);
        assert_eq!(result.status, BatchStatus::Completed);
    }

    #[test]
    fn test_execute_batch_single_operation() {
        let (env, client, admin) = setup_initialized();

        let mut ops = Vec::new(&env);
        ops.push_back(create_operation(&env, BatchOpType::Delete));

        let result = client.execute_batch(&admin, &ops);

        assert_eq!(result.batch_id, 1);
        assert_eq!(result.total_operations, 1);
        assert_eq!(result.successful, 1);
    }

    #[test]
    fn test_execute_batch_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);

        let ops = Vec::new(&env);

        let result = client.try_execute_batch(&non_admin, &ops);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_get_batch_result() {
        let (env, client, admin) = setup_initialized();

        let mut ops = Vec::new(&env);
        ops.push_back(create_operation(&env, BatchOpType::Create));

        client.execute_batch(&admin, &ops);

        let result = client.get_batch_result(&1);
        assert!(result.is_some());
    }

    #[test]
    fn test_get_batch_result_not_found() {
        let (env, client) = setup();
        assert!(client.get_batch_result(&999).is_none());
    }

    #[test]
    fn test_total_batches() {
        let (env, client, admin) = setup_initialized();

        assert_eq!(client.total_batches(), 0);

        let ops = Vec::new(&env);
        client.execute_batch(&admin, &ops);

        assert_eq!(client.total_batches(), 1);
    }
}
