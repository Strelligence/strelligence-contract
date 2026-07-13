#[cfg(test)]
mod bench {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    use crate::contract::RecurringRegistryContract;
    use crate::contract::RecurringRegistryContractClient;
    use crate::types::{Frequency, SubscriptionType};

    fn setup<'a>() -> (Env, RecurringRegistryContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, RecurringRegistryContract);
        let client = RecurringRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn bench_create_subscription() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let start = env.ledger().sequence();

        for i in 0..100 {
            client.create_subscription(
                &owner,
                &owner,
                &String::from_str(&env, "Merchant"),
                &None,
                &Frequency::Monthly,
                &SubscriptionType::Subscription,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer"),
                &10_000_000,
                &(1000 + i as u64),
                &false,
                &None,
            );
        }

        let end = env.ledger().sequence();
        let elapsed = end - start;
        assert!(
            elapsed < 100,
            "100 subscriptions created in {} ledgers (expected < 100)",
            elapsed
        );
        assert_eq!(client.total_subscriptions(), 100);
    }

    #[test]
    fn bench_list_wallet_subscriptions() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        for _ in 0..50 {
            client.create_subscription(
                &owner,
                &owner,
                &String::from_str(&env, "Merchant"),
                &None,
                &Frequency::Monthly,
                &SubscriptionType::Subscription,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer"),
                &10_000_000,
                &1000,
                &false,
                &None,
            );
        }

        let start = env.ledger().sequence();
        let _ids = client.list_wallet_subscriptions(&owner);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "List 50 subscriptions took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_get_subscription() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Merchant"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let start = env.ledger().sequence();
        for _ in 0..100 {
            let _ = client.get_subscription(&id);
        }
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "100 reads took {} ledgers (expected < 10)",
            elapsed
        );
    }
}
