#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    use automation_rules::AutomationRulesContract;
    use automation_rules::AutomationRulesContractClient;
    use automation_rules::{RuleTrigger, RuleType};
    use metadata_registry::MetadataRegistryContract;
    use metadata_registry::MetadataRegistryContractClient;
    use metadata_registry::{TransactionCategory, TransactionSentiment};
    use recurring_registry::RecurringRegistryContract;
    use recurring_registry::RecurringRegistryContractClient;
    use recurring_registry::{Frequency, SubscriptionStatus, SubscriptionType};

    fn setup<'a>() -> (
        Env,
        RecurringRegistryContractClient<'a>,
        MetadataRegistryContractClient<'a>,
        AutomationRulesContractClient<'a>,
    ) {
        let env = Env::default();
        let recurring_id = env.register_contract(None, RecurringRegistryContract);
        let metadata_id = env.register_contract(None, MetadataRegistryContract);
        let automation_id = env.register_contract(None, AutomationRulesContract);

        let recurring_client = RecurringRegistryContractClient::new(&env, &recurring_id);
        let metadata_client = MetadataRegistryContractClient::new(&env, &metadata_id);
        let automation_client = AutomationRulesContractClient::new(&env, &automation_id);

        (env, recurring_client, metadata_client, automation_client)
    }

    #[test]
    fn test_subscription_update_with_metadata_sync() {
        let (env, recurring, metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let sub_id = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &15_000_000,
            &1000,
            &true,
            &None,
        );

        let tx_hash = String::from_str(&env, "tx_update_001");
        metadata.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &true,
            &Some(sub_id),
            &85,
        );

        recurring.update_subscription(
            &owner,
            &sub_id,
            &Some(String::from_str(&env, "Netflix Premium")),
            &None,
            &Some(20_000_000),
            &None,
            &None,
        );

        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.merchant, String::from_str(&env, "Netflix Premium"));
        assert_eq!(sub.amount, 20_000_000);

        let meta = metadata.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.recurring_id, Some(sub_id));
    }

    #[test]
    fn test_subscription_cancel_affects_active_list() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let sub1 = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub2 = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Spotify"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &5_000_000,
            &2000,
            &false,
            &None,
        );

        let sub3 = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Adobe"),
            &None,
            &Frequency::Annually,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &50_000_000,
            &3000,
            &false,
            &None,
        );

        assert_eq!(
            recurring.list_active_subscriptions(&owner, &0, &10).len(),
            3
        );

        recurring.cancel_subscription(&owner, &sub2);
        assert_eq!(
            recurring.list_active_subscriptions(&owner, &0, &10).len(),
            2
        );

        recurring.pause_subscription(&owner, &sub3);
        assert_eq!(
            recurring.list_active_subscriptions(&owner, &0, &10).len(),
            1
        );

        recurring.resume_subscription(&owner, &sub3);
        assert_eq!(
            recurring.list_active_subscriptions(&owner, &0, &10).len(),
            2
        );
    }

    #[test]
    fn test_multiple_payment_confirmations() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let sub_id = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &15_000_000,
            &1000,
            &true,
            &None,
        );

        recurring.confirm_payment(&owner, &sub_id, &1000, &2000);
        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.last_payment_ledger, 1000);
        assert_eq!(sub.next_payment_ledger, 2000);

        recurring.confirm_payment(&owner, &sub_id, &2000, &3000);
        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.last_payment_ledger, 2000);
        assert_eq!(sub.next_payment_ledger, 3000);

        recurring.confirm_payment(&owner, &sub_id, &3000, &4000);
        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.last_payment_ledger, 3000);
        assert_eq!(sub.next_payment_ledger, 4000);
    }

    #[test]
    fn test_automation_rule_with_subscription_trigger() {
        let (env, recurring, _metadata, automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let sub_id = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &15_000_000,
            &1000,
            &true,
            &None,
        );

        let rule_id = automation.create_rule(
            &owner,
            &RuleType::Alert,
            &RuleTrigger::OnBalanceBelow,
            &String::from_str(&env, "Low balance alert"),
            &String::from_str(&env, r#"{"threshold":100000000}"#),
            &String::from_str(&env, r#"{"notify":true}"#),
        );

        automation.record_execution(&Address::generate(&env), &rule_id);
        automation.record_execution(&Address::generate(&env), &rule_id);
        automation.record_execution(&Address::generate(&env), &rule_id);

        let rule = automation.get_rule(&rule_id).unwrap();
        assert_eq!(rule.execution_count, 3);

        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.amount, 15_000_000);
    }

    #[test]
    fn test_concurrent_wallet_operations() {
        let (env, recurring, metadata, _automation) = setup();
        env.mock_all_auths();

        let wallet_a = Address::generate(&env);
        let wallet_b = Address::generate(&env);

        let sub_a1 = recurring.create_subscription(
            &wallet_a,
            &wallet_a,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub_a2 = recurring.create_subscription(
            &wallet_a,
            &wallet_a,
            &String::from_str(&env, "Spotify"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &5_000_000,
            &2000,
            &false,
            &None,
        );

        let sub_b1 = recurring.create_subscription(
            &wallet_b,
            &wallet_b,
            &String::from_str(&env, "Adobe"),
            &None,
            &Frequency::Annually,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &50_000_000,
            &3000,
            &false,
            &None,
        );

        metadata.add_metadata(
            &wallet_a,
            &String::from_str(&env, "tx_a1"),
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &true,
            &Some(sub_a1),
            &90,
        );

        metadata.add_metadata(
            &wallet_b,
            &String::from_str(&env, "tx_b1"),
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &true,
            &Some(sub_b1),
            &85,
        );

        let ids_a = recurring.list_wallet_subscriptions(&wallet_a, &0, &10);
        let ids_b = recurring.list_wallet_subscriptions(&wallet_b, &0, &10);

        assert_eq!(ids_a.len(), 2);
        assert_eq!(ids_b.len(), 1);

        let meta_a = metadata.get_wallet_metadata(&wallet_a);
        let meta_b = metadata.get_wallet_metadata(&wallet_b);

        assert_eq!(meta_a.len(), 1);
        assert_eq!(meta_b.len(), 1);

        recurring.cancel_subscription(&wallet_a, &sub_a1);
        let active_a = recurring.list_active_subscriptions(&wallet_a, &0, &10);
        assert_eq!(active_a.len(), 1);
    }

    #[test]
    fn test_subscription_with_all_frequency_types() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let frequencies = [
            Frequency::Daily,
            Frequency::Weekly,
            Frequency::BiWeekly,
            Frequency::Monthly,
            Frequency::Quarterly,
            Frequency::Annually,
            Frequency::Custom,
        ];

        for (i, freq) in frequencies.iter().enumerate() {
            recurring.create_subscription(
                &owner,
                &owner,
                &String::from_str(&env, &format!("Merchant{}", i)),
                &None,
                freq,
                &SubscriptionType::Subscription,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer1"),
                &10_000_000,
                &(1000 + i as u64),
                &false,
                &None,
            );
        }

        let ids = recurring.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 7);

        let active = recurring.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 7);

        assert_eq!(recurring.total_subscriptions(), 7);
    }

    #[test]
    fn test_subscription_with_all_type_variants() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let types = [
            SubscriptionType::Subscription,
            SubscriptionType::Payroll,
            SubscriptionType::Income,
            SubscriptionType::Savings,
            SubscriptionType::Bill,
            SubscriptionType::Investment,
            SubscriptionType::Transfer,
            SubscriptionType::Other,
        ];

        for (i, sub_type) in types.iter().enumerate() {
            recurring.create_subscription(
                &owner,
                &owner,
                &String::from_str(&env, &format!("Type{}", i)),
                &None,
                &Frequency::Monthly,
                sub_type,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer1"),
                &10_000_000,
                &(1000 + i as u64),
                &false,
                &None,
            );
        }

        let ids = recurring.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 8);

        assert_eq!(recurring.total_subscriptions(), 8);
    }

    #[test]
    fn test_batch_metadata_operations() {
        let (env, _recurring, metadata, _automation) = setup();
        env.mock_all_auths();

        let wallet = Address::generate(&env);

        for i in 0..10 {
            let tx_hash = String::from_str(&env, &format!("tx_{:03}", i));
            metadata.add_metadata(
                &wallet,
                &tx_hash,
                &TransactionCategory::Subscription,
                &TransactionSentiment::Negative,
                &Vec::new(&env),
                &None,
                &None,
                &None,
                &false,
                &None,
                &80,
            );
        }

        let hashes = metadata.get_wallet_metadata(&wallet);
        assert_eq!(hashes.len(), 10);
    }

    #[test]
    fn test_subscription_amount_boundaries() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let sub_min = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "MinAmount"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &1,
            &1000,
            &false,
            &None,
        );

        let sub_max = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "MaxAmount"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &(i128::MAX / 2),
            &2000,
            &false,
            &None,
        );

        let min_sub = recurring.get_subscription(&sub_min).unwrap();
        assert_eq!(min_sub.amount, 1);

        let max_sub = recurring.get_subscription(&sub_max).unwrap();
        assert_eq!(max_sub.amount, i128::MAX / 2);
    }

    #[test]
    fn test_subscription_with_long_labels() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let long_label = String::from_str(
            &env,
            "This is a very long custom label for my Netflix subscription that I use for streaming movies and TV shows",
        );

        let sub_id = recurring.create_subscription(
            &owner,
            &owner,
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &15_000_000,
            &1000,
            &true,
            &Some(long_label.clone()),
        );

        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.custom_label, Some(long_label));
    }
}
