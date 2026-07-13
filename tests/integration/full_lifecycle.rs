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
    fn test_subscription_lifecycle_with_metadata() {
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

        let tx = String::from_str(&env, "tx_001");
        metadata.add_metadata(
            &owner,
            &tx,
            &TransactionCategory::Subscription,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &true,
            &Some(sub_id),
            &90,
        );

        recurring.pause_subscription(&owner, &sub_id);
        assert_eq!(
            recurring.get_subscription(&sub_id).unwrap().status,
            SubscriptionStatus::Paused
        );

        recurring.resume_subscription(&owner, &sub_id);
        assert_eq!(
            recurring.get_subscription(&sub_id).unwrap().status,
            SubscriptionStatus::Active
        );

        recurring.confirm_payment(&owner, &sub_id, &5000, &6000);

        recurring.cancel_subscription(&owner, &sub_id);
        assert_eq!(
            recurring.get_subscription(&sub_id).unwrap().status,
            SubscriptionStatus::Cancelled
        );

        let meta = metadata.get_metadata(&tx).unwrap();
        assert_eq!(meta.recurring_id, Some(sub_id));
    }

    #[test]
    fn test_automation_rule_lifecycle() {
        let (env, _recurring, _metadata, automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let rule_id = automation.create_rule(
            &owner,
            &RuleType::AutoSave,
            &RuleTrigger::OnIncomingPayment,
            &String::from_str(&env, "Save 10%"),
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "{}"),
        );

        automation.pause_rule(&owner, &rule_id);
        assert_eq!(
            automation.get_rule(&rule_id).unwrap().status,
            RuleStatus::Paused
        );

        automation.resume_rule(&owner, &rule_id);
        assert_eq!(
            automation.get_rule(&rule_id).unwrap().status,
            RuleStatus::Active
        );

        automation.record_execution(&Address::generate(&env), &rule_id);
        automation.record_execution(&Address::generate(&env), &rule_id);

        let rule = automation.get_rule(&rule_id).unwrap();
        assert_eq!(rule.execution_count, 2);

        automation.delete_rule(&owner, &rule_id);
        assert_eq!(
            automation.get_rule(&rule_id).unwrap().status,
            RuleStatus::Deleted
        );
    }

    #[test]
    fn test_category_filtering() {
        let (env, _recurring, metadata, _automation) = setup();
        env.mock_all_auths();

        let wallet = Address::generate(&env);

        metadata.add_metadata(
            &wallet,
            &String::from_str(&env, "tx1"),
            &TransactionCategory::Income,
            &TransactionSentiment::Positive,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &90,
        );

        metadata.add_metadata(
            &wallet,
            &String::from_str(&env, "tx2"),
            &TransactionCategory::Expense,
            &TransactionSentiment::Negative,
            &Vec::new(&env),
            &None,
            &None,
            &None,
            &false,
            &None,
            &85,
        );

        metadata.add_metadata(
            &wallet,
            &String::from_str(&env, "tx3"),
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

        let income = metadata.get_metadata_by_category(&wallet, &TransactionCategory::Income);
        assert_eq!(income.len(), 2);

        let expense = metadata.get_metadata_by_category(&wallet, &TransactionCategory::Expense);
        assert_eq!(expense.len(), 1);

        let swaps = metadata.get_metadata_by_category(&wallet, &TransactionCategory::Swap);
        assert_eq!(swaps.len(), 0);
    }

    #[test]
    fn test_active_rules_filtering() {
        let (env, _recurring, _metadata, automation) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let _id1 = automation.create_rule(
            &owner,
            &RuleType::AutoSave,
            &RuleTrigger::OnIncomingPayment,
            &String::from_str(&env, "Rule 1"),
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "{}"),
        );

        let id2 = automation.create_rule(
            &owner,
            &RuleType::Budget,
            &RuleTrigger::OnCategorySpend,
            &String::from_str(&env, "Rule 2"),
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "{}"),
        );

        let _id3 = automation.create_rule(
            &owner,
            &RuleType::Alert,
            &RuleTrigger::OnBalanceBelow,
            &String::from_str(&env, "Rule 3"),
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "{}"),
        );

        automation.pause_rule(&owner, &id2);

        let active = automation.list_active_rules(&owner);
        assert_eq!(active.len(), 2);

        let all = automation.list_wallet_rules(&owner);
        assert_eq!(all.len(), 3);
    }
}
