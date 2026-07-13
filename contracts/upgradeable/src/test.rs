#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env};

    use crate::contract::UpgradeableContract;
    use crate::contract::UpgradeableContractClient;
    use crate::errors::ContractError;

    fn setup<'a>() -> (Env, UpgradeableContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, UpgradeableContract);
        let client = UpgradeableContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn test_initialize() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_version(), 1);
        assert!(client.get_wasm_hash().is_some());
    }

    #[test]
    fn test_initialize_twice_panics() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let result = client.try_initialize(&admin);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_version_initial() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_version(), 1);
    }

    #[test]
    fn test_upgrade_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let new_hash = Bytes::from_array(&env, &[2u8; 32]);
        client.upgrade(&admin, &new_hash);

        assert_eq!(client.get_version(), 2);
        assert_eq!(client.get_wasm_hash(), Some(new_hash));
    }

    #[test]
    fn test_upgrade_increments_version() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let hash1 = Bytes::from_array(&env, &[2u8; 32]);
        client.upgrade(&admin, &hash1);
        assert_eq!(client.get_version(), 2);

        let hash2 = Bytes::from_array(&env, &[3u8; 32]);
        client.upgrade(&admin, &hash2);
        assert_eq!(client.get_version(), 3);
    }

    #[test]
    fn test_upgrade_same_hash_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let current_hash = client.get_wasm_hash().unwrap();
        let result = client.try_upgrade(&admin, &current_hash);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::SameWasmHash
        );
    }

    #[test]
    fn test_upgrade_unauthorized_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let attacker = Address::generate(&env);
        let new_hash = Bytes::from_array(&env, &[2u8; 32]);
        let result = client.try_upgrade(&attacker, &new_hash);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::Unauthorized
        );
    }

    #[test]
    fn test_get_admin() {
        let (env, client) = setup();
        env.mock_all_auths();

        assert_eq!(client.get_admin(), None);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_admin(), Some(admin));
    }

    #[test]
    fn test_get_wasm_hash() {
        let (env, client) = setup();
        env.mock_all_auths();

        assert_eq!(client.get_wasm_hash(), None);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let hash = client.get_wasm_hash();
        assert!(hash.is_some());
        assert_eq!(hash.unwrap().len(), 32);
    }
}
