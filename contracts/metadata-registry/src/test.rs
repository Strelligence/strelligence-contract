#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    use crate::contract::MetadataRegistryContract;
    use crate::contract::MetadataRegistryContractClient;
    use crate::errors::ContractError;
    use crate::types::{TransactionCategory, TransactionSentiment};

    fn setup<'a>() -> (Env, MetadataRegistryContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, MetadataRegistryContract);
        let client = MetadataRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn sample_tags(env: &Env) -> Vec<String> {
        let mut tags = Vec::new(env);
        tags.push_back(String::from_str(env, "food"));
        tags.push_back(String::from_str(env, "recurring"));
        tags
    }

    // ─────────────────────────────────────────────────────────────────────────
    // add_metadata
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_add_metadata_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.try_add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &sample_tags(&env),
            &Some(String::from_str(&env, "Netflix Payment")),
            &Some(String::from_str(&env, "Monthly subscription")),
            &Some(String::from_str(&env, "Netflix")),
            &true,
            &Some(42),
            &95,
        );

        assert!(result.is_ok());

        let meta = client.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.category, TransactionCategory::Expense);
        assert_eq!(meta.sentiment, TransactionSentiment::Negative);
        assert_eq!(meta.ai_confidence, 95);
        assert_eq!(meta.is_recurring, true);
        assert_eq!(meta.recurring_id, Some(42));
    }

    #[test]
    fn test_add_metadata_already_exists_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        client.add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &sample_tags(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &95,
        );

        let result = client.try_add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &80,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyExists);
    }

    #[test]
    fn test_add_metadata_invalid_confidence_over_100() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.try_add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &101,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidConfidence
        );
    }

    #[test]
    fn test_add_metadata_confidence_exactly_100_ok() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.try_add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &100,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_metadata_confidence_zero_ok() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        let result = client.try_add_metadata(
            &caller,
            &tx_hash,
            &TransactionCategory::Unknown,
            &TransactionSentiment::Neutral,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &0,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_metadata_updates_wallet_index() {
        let (env, client) = setup();
        env.mock_all_auths();

        let caller = Address::generate(&env);
        let tx_hash1 = String::from_str(&env, "abc123");
        let tx_hash2 = String::from_str(&env, "def456");

        client.add_metadata(
            &caller,
            &tx_hash1,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        client.add_metadata(
            &caller,
            &tx_hash2,
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &85,
        );

        let hashes = client.get_wallet_metadata(&caller);
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes.get_unchecked(0), tx_hash1);
        assert_eq!(hashes.get_unchecked(1), tx_hash2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // update_metadata
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_update_metadata_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        client.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        client.update_metadata(
            &owner,
            &tx_hash,
            &Some(TransactionCategory::Income),
            &Some(TransactionSentiment::Positive),
            &Some(sample_tags(&env)),
            &Some(String::from_str(&env, "Salary Payment")),
            &Some(String::from_str(&env, "Monthly salary")),
            &Some(String::from_str(&env, "Acme Corp")),
        );

        let meta = client.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.category, TransactionCategory::Income);
        assert_eq!(meta.sentiment, TransactionSentiment::Positive);
        assert_eq!(
            meta.label,
            Some(String::from_str(&env, "Salary Payment"))
        );
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn test_update_metadata_not_found_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "nonexistent");

        let result = client.try_update_metadata(
            &owner,
            &tx_hash,
            &Some(TransactionCategory::Income),
            &None,
            &None,
            &None,
            &None,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::MetadataNotFound
        );
    }

    #[test]
    fn test_update_metadata_not_owner_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        client.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        let result = client.try_update_metadata(
            &attacker,
            &tx_hash,
            &Some(TransactionCategory::Income),
            &None,
            &None,
            &None,
            &None,
            &None,
        );

        assert!(result.is_ok());
        let meta = client.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.category, TransactionCategory::Income);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_metadata
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_metadata_returns_some_for_existing() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "abc123");

        client.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        let meta = client.get_metadata(&tx_hash);
        assert!(meta.is_some());
    }

    #[test]
    fn test_get_metadata_returns_none_for_missing() {
        let (env, client) = setup();

        let result = client.get_metadata(&String::from_str(&env, "missing"));
        assert!(result.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_wallet_metadata
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_wallet_metadata_returns_all_tx_hashes() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx1"),
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx2"),
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &85,
        );

        let hashes = client.get_wallet_metadata(&owner);
        assert_eq!(hashes.len(), 2);
    }

    #[test]
    fn test_get_wallet_metadata_empty_for_new_wallet() {
        let (env, client) = setup();

        let owner = Address::generate(&env);
        let hashes = client.get_wallet_metadata(&owner);
        assert_eq!(hashes.len(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_metadata_by_category
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_metadata_by_category_filters_correctly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx1"),
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx2"),
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &85,
        );

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx3"),
            &TransactionCategory::Expense,
            &TransactionSentiment::Neutral,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &80,
        );

        let expenses = client.get_metadata_by_category(&owner, &TransactionCategory::Expense);
        assert_eq!(expenses.len(), 2);

        let income = client.get_metadata_by_category(&owner, &TransactionCategory::Income);
        assert_eq!(income.len(), 1);
    }

    #[test]
    fn test_get_metadata_by_category_empty_for_no_matches() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        client.add_metadata(
            &owner,
            &String::from_str(&env, "tx1"),
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        let swaps = client.get_metadata_by_category(&owner, &TransactionCategory::Swap);
        assert_eq!(swaps.len(), 0);
    }

    #[test]
    fn test_get_metadata_by_category_empty_wallet() {
        let (env, client) = setup();

        let owner = Address::generate(&env);
        let results =
            client.get_metadata_by_category(&owner, &TransactionCategory::Expense);
        assert_eq!(results.len(), 0);
    }
}
