#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    use automation_rules::AutomationRulesContract;
    use automation_rules::AutomationRulesContractClient;
    use automation_rules::{RuleStatus, RuleTrigger, RuleType};
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
    fn test_subscription_then_metadata() {
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
            &10_000_000,
            &1000,
            &true,
            &Some(String::from_str(&env, "Netflix Subscription")),
        );

        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);

        let tx_hash = String::from_str(&env, "tx_abc123");
        metadata.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &Some(String::from_str(&env, "Netflix Payment")),
            &Some(String::from_str(&env, "Monthly streaming service")),
            &Some(String::from_str(&env, "Netflix")),
            &true,
            &Some(sub_id),
            &95,
        );

        let meta = metadata.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.category, TransactionCategory::Subscription);
        assert!(meta.is_recurring);
        assert_eq!(meta.recurring_id, Some(sub_id));

        recurring.confirm_payment(&owner, &sub_id, &5000, &6000);
        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.last_payment_ledger, 5000);
    }

    #[test]
    fn test_automation_rule_with_metadata_update() {
        let (env, _recurring, metadata, automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let rule_id = automation.create_rule(
            &owner,
            &RuleType::AutoSave,
            &RuleTrigger::OnIncomingPayment,
            &String::from_str(&env, "Auto-save 10% of income"),
            &String::from_str(&env, r#"{"percentage":10}"#),
            &String::from_str(&env, r#"{"dest":"G..."}"#),
        );

        let rule = automation.get_rule(&rule_id).unwrap();
        assert_eq!(rule.status, RuleStatus::Active);

        let tx_hash = String::from_str(&env, "tx_salary_001");
        metadata.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &Some(String::from_str(&env, "Monthly Salary")),
            &Some(String::from_str(&env, "Acme Corp paycheck")),
            &Some(String::from_str(&env, "Acme Corp")),
            &false,
            &None,
            &100,
        );

        automation.record_execution(&Address::generate(&env), &rule_id);
        let rule = automation.get_rule(&rule_id).unwrap();
        assert_eq!(rule.execution_count, 1);

        metadata.update_metadata(
            &owner,
            &tx_hash,
            &None,
            &None,
            &Some(Vec::new(&env)),
            &None,
            &Some(String::from_str(&env, "January salary from Acme")),
            &None,
        );

        let meta = metadata.get_metadata(&tx_hash).unwrap();
        assert_eq!(
            meta.notes,
            Some(String::from_str(&env, "January salary from Acme"))
        );
    }

    #[test]
    fn test_multi_wallet_operations() {
        let (env, recurring, metadata, automation) = setup();
        env.mock_all_auths();

        let wallet_a = Address::generate(&env);
        let wallet_b = Address::generate(&env);

        let sub_a = recurring.create_subscription(
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

        let sub_b = recurring.create_subscription(
            &wallet_b,
            &wallet_b,
            &String::from_str(&env, "Spotify"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &5_000_000,
            &1000,
            &false,
            &None,
        );

        let ids_a = recurring.list_wallet_subscriptions(&wallet_a, &0, &10);
        let ids_b = recurring.list_wallet_subscriptions(&wallet_b, &0, &10);

        assert_eq!(ids_a.len(), 1);
        assert_eq!(ids_b.len(), 1);
        assert_eq!(ids_a.get_unchecked(0), sub_a);
        assert_eq!(ids_b.get_unchecked(0), sub_b);

        let meta_a = String::from_str(&env, "tx_a_001");
        let meta_b = String::from_str(&env, "tx_b_001");

        metadata.add_metadata(
            &wallet_a,
            &meta_a,
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

        metadata.add_metadata(
            &wallet_b,
            &meta_b,
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

        let hashes_a = metadata.get_wallet_metadata(&wallet_a);
        let hashes_b = metadata.get_wallet_metadata(&wallet_b);

        assert_eq!(hashes_a.len(), 1);
        assert_eq!(hashes_b.len(), 1);

        let rule_a = automation.create_rule(
            &wallet_a,
            &RuleType::Budget,
            &RuleTrigger::OnCategorySpend,
            &String::from_str(&env, "Limit streaming spend"),
            &String::from_str(&env, r#"{"category":"subscription","max":50000000}"#),
            &String::from_str(&env, r#"{"alert":true}"#),
        );

        let rules_a = automation.list_wallet_rules(&wallet_a);
        let rules_b = automation.list_wallet_rules(&wallet_b);

        assert_eq!(rules_a.len(), 1);
        assert_eq!(rules_b.len(), 0);
        assert_eq!(rules_a.get_unchecked(0), rule_a);
    }

    #[test]
    fn test_batch_subscription_operations() {
        let (env, recurring, _metadata, _automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let merchants = ["Netflix", "Spotify", "Adobe", "Figma", "Notion"];

        for merchant in merchants.iter() {
            recurring.create_subscription(
                &owner,
                &owner,
                &String::from_str(&env, merchant),
                &None,
                &Frequency::Monthly,
                &SubscriptionType::Subscription,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer1"),
                &5_000_000,
                &1000,
                &true,
                &None,
            );
        }

        let ids = recurring.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 5);

        let active = recurring.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 5);

        assert_eq!(recurring.total_subscriptions(), 5);

        recurring.cancel_subscription(&owner, &ids.get_unchecked(0));
        recurring.pause_subscription(&owner, &ids.get_unchecked(1));

        let active = recurring.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_full_user_flow() {
        let (env, recurring, metadata, automation) = setup();
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
            &Some(String::from_str(&env, "My Netflix")),
        );

        let tx_hash = String::from_str(&env, "tx_netflix_jan");
        metadata.add_metadata(
            &owner,
            &tx_hash,
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &soroban_sdk::vec![
                &env,
                String::from_str(&env, "streaming"),
                String::from_str(&env, "monthly"),
            ],
            &Some(String::from_str(&env, "Netflix Jan")),
            &Some(String::from_str(&env, "Jan 2025 payment")),
            &Some(String::from_str(&env, "Netflix")),
            &true,
            &Some(sub_id),
            &98,
        );

        automation.create_rule(
            &owner,
            &RuleType::Budget,
            &RuleTrigger::OnCategorySpend,
            &String::from_str(&env, "Streaming budget alert"),
            &String::from_str(&env, r#"{"category":"subscription","max":30000000}"#),
            &String::from_str(&env, r#"{"alert":true,"channel":"webhook"}"#),
        );

        recurring.confirm_payment(&owner, &sub_id, &2000, &3000);

        automation.record_execution(&Address::generate(&env), &1);

        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.last_payment_ledger, 2000);

        let meta = metadata.get_metadata(&tx_hash).unwrap();
        assert_eq!(meta.category, TransactionCategory::Subscription);
        assert_eq!(meta.ai_confidence, 98);

        let rules = automation.list_active_rules(&owner);
        assert_eq!(rules.len(), 1);

        recurring.cancel_subscription(&owner, &sub_id);
        let sub = recurring.get_subscription(&sub_id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
    }
}
