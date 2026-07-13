#[cfg(test)]
mod bench {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    use crate::contract::MetadataRegistryContract;
    use crate::contract::MetadataRegistryContractClient;
    use crate::types::{TransactionCategory, TransactionSentiment};

    fn setup<'a>() -> (Env, MetadataRegistryContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, MetadataRegistryContract);
        let client = MetadataRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn bench_add_metadata_single() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "bench_tx_001");

        let start = env.ledger().sequence();

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
            &80,
        );

        let end = env.ledger().sequence();
        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "Single metadata add took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_get_wallet_metadata() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "bench_tx_002");
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
            &80,
        );

        let start = env.ledger().sequence();
        let _hashes = client.get_wallet_metadata(&owner);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "List metadata took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_get_metadata_by_category() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let tx_hash = String::from_str(&env, "bench_tx_003");
        client.add_metadata(
            &owner,
            &tx_hash,
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

        let start = env.ledger().sequence();
        let _results = client.get_metadata_by_category(&owner, &TransactionCategory::Expense);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "Filter metadata took {} ledgers (expected < 10)",
            elapsed
        );
    }
}
