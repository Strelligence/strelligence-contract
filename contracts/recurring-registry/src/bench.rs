#[cfg(test)]
mod bench {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    use crate::contract::RecurringRegistryContract;
    use crate::contract::RecurringRegistryContractClient;
    use crate::types::{Frequency, SubscriptionStatus, SubscriptionType};

    fn setup<'a>() -> (Env, RecurringRegistryContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, RecurringRegistryContract);
        let client = RecurringRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Gas Usage Benchmarks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bench_create_subscription_gas() {
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
    fn bench_update_subscription_gas() {
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
        for i in 0..100 {
            client.update_subscription(
                &owner,
                &id,
                &None,
                &None,
                &Some(10_000_000 + i as i128),
                &None,
                &None,
            );
        }
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "100 updates took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_cancel_subscription_gas() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let mut ids = Vec::new(&env);
        for i in 0..100 {
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
                &(1000 + i as u64),
                &false,
                &None,
            );
            ids.push_back(id);
        }

        let start = env.ledger().sequence();
        for i in 0..100 {
            client.cancel_subscription(&owner, &ids.get_unchecked(i));
        }
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "100 cancels took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_pause_resume_gas() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let mut ids = Vec::new(&env);
        for i in 0..50 {
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
                &(1000 + i as u64),
                &false,
                &None,
            );
            ids.push_back(id);
        }

        let start = env.ledger().sequence();
        for i in 0..50 {
            client.pause_subscription(&owner, &ids.get_unchecked(i));
        }
        for i in 0..50 {
            client.resume_subscription(&owner, &ids.get_unchecked(i));
        }
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 20,
            "50 pause+resume cycles took {} ledgers (expected < 20)",
            elapsed
        );
    }

    #[test]
    fn bench_confirm_payment_gas() {
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
        for i in 0..100 {
            client.confirm_payment(&owner, &id, &(1000 + i as u64), &(2000 + i as u64));
        }
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "100 payment confirmations took {} ledgers (expected < 10)",
            elapsed
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Execution Time Benchmarks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bench_list_wallet_subscriptions_time() {
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
        let _ids = client.list_wallet_subscriptions(&owner, &0, &50);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "List 50 subscriptions took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_get_subscription_time() {
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

    #[test]
    fn bench_list_active_subscriptions_time() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let mut ids = Vec::new(&env);
        for i in 0..50 {
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
                &(1000 + i as u64),
                &false,
                &None,
            );
            ids.push_back(id);
        }

        for i in 0..25 {
            client.cancel_subscription(&owner, &ids.get_unchecked(i));
        }

        let start = env.ledger().sequence();
        let _active = client.list_active_subscriptions(&owner, &0, &50);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "List 25 active (from 50 total) took {} ledgers (expected < 10)",
            elapsed
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Storage Cost Benchmarks
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bench_storage_cost_per_subscription() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let start = env.ledger().sequence();
        for i in 0..10 {
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
        let per_subscription = elapsed / 10;
        assert!(
            per_subscription < 5,
            "Storage cost per subscription: {} ledgers (expected < 5)",
            per_subscription
        );
    }

    #[test]
    fn bench_storage_index_cost() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        for _ in 0..100 {
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
        let _ids = client.list_wallet_subscriptions(&owner, &0, &100);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 5,
            "Index read cost for 100 subscriptions: {} ledgers (expected < 5)",
            elapsed
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Regression Testing
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn regression_create_performance() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let iterations = 100;
        let start = env.ledger().sequence();

        for i in 0..iterations {
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
            "REGRESSION: Create {} subscriptions took {} ledgers (expected < 100)",
            iterations,
            elapsed
        );

        assert_eq!(client.total_subscriptions(), iterations as u64);
    }

    #[test]
    fn regression_read_performance() {
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

        let iterations = 1000;
        let start = env.ledger().sequence();

        for _ in 0..iterations {
            let _ = client.get_subscription(&id);
        }

        let end = env.ledger().sequence();
        let elapsed = end - start;

        assert!(
            elapsed < 10,
            "REGRESSION: {} reads took {} ledgers (expected < 10)",
            iterations,
            elapsed
        );
    }

    #[test]
    fn regression_mixed_operations() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let start = env.ledger().sequence();

        let mut ids = Vec::new(&env);
        for i in 0..25 {
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
                &(1000 + i as u64),
                &false,
                &None,
            );
            ids.push_back(id);
        }

        for i in 0..25 {
            client.update_subscription(
                &owner,
                &ids.get_unchecked(i),
                &None,
                &None,
                &Some(10_000_000 + i as i128),
                &None,
                &None,
            );
        }

        for i in 0..25 {
            client.confirm_payment(
                &owner,
                &ids.get_unchecked(i),
                &(1000 + i as u64),
                &(2000 + i as u64),
            );
        }

        for i in 0..10 {
            client.cancel_subscription(&owner, &ids.get_unchecked(i));
        }

        for i in 10..20 {
            client.pause_subscription(&owner, &ids.get_unchecked(i));
        }

        for i in 10..20 {
            client.resume_subscription(&owner, &ids.get_unchecked(i));
        }

        let end = env.ledger().sequence();
        let elapsed = end - start;

        assert!(
            elapsed < 100,
            "REGRESSION: Mixed operations took {} ledgers (expected < 100)",
            elapsed
        );

        let active = client.list_active_subscriptions(&owner, &0, &20);
        assert_eq!(active.len(), 15);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Benchmark Reporting
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bench_report_comprehensive() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let iterations = 100;

        let start_create = env.ledger().sequence();
        let mut ids = Vec::new(&env);
        for i in 0..iterations {
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
                &(1000 + i as u64),
                &false,
                &None,
            );
            ids.push_back(id);
        }
        let end_create = env.ledger().sequence();

        let start_update = env.ledger().sequence();
        for i in 0..iterations {
            client.update_subscription(
                &owner,
                &ids.get_unchecked(i),
                &None,
                &None,
                &Some(10_000_000 + i as i128),
                &None,
                &None,
            );
        }
        let end_update = env.ledger().sequence();

        let start_read = env.ledger().sequence();
        for i in 0..iterations {
            let _ = client.get_subscription(&ids.get_unchecked(i));
        }
        let end_read = env.ledger().sequence();

        let start_list = env.ledger().sequence();
        let _ids = client.list_wallet_subscriptions(&owner, &0, &iterations);
        let end_list = env.ledger().sequence();

        let start_cancel = env.ledger().sequence();
        for i in 0..iterations {
            client.cancel_subscription(&owner, &ids.get_unchecked(i));
        }
        let end_cancel = env.ledger().sequence();

        let create_time = end_create - start_create;
        let update_time = end_update - start_update;
        let read_time = end_read - start_read;
        let list_time = end_list - start_list;
        let cancel_time = end_cancel - start_cancel;

        assert!(
            create_time < 100,
            "Create benchmark failed: {} ledgers",
            create_time
        );
        assert!(
            update_time < 10,
            "Update benchmark failed: {} ledgers",
            update_time
        );
        assert!(
            read_time < 10,
            "Read benchmark failed: {} ledgers",
            read_time
        );
        assert!(
            list_time < 10,
            "List benchmark failed: {} ledgers",
            list_time
        );
        assert!(
            cancel_time < 10,
            "Cancel benchmark failed: {} ledgers",
            cancel_time
        );

        assert_eq!(client.total_subscriptions(), iterations as u64);
    }
}
