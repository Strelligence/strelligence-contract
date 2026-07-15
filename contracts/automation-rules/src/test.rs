#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String};

    use crate::contract::AutomationRulesContract;
    use crate::contract::AutomationRulesContractClient;
    use crate::errors::ContractError;
    use crate::types::{RuleStatus, RuleTrigger, RuleType};

    fn setup<'a>() -> (Env, AutomationRulesContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, AutomationRulesContract);
        let client = AutomationRulesContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn create_sample_rule(
        client: &AutomationRulesContractClient,
        env: &Env,
        owner: &Address,
    ) -> u64 {
        client.create_rule(
            owner,
            &RuleType::AutoSave,
            &RuleTrigger::OnIncomingPayment,
            &String::from_str(env, "Auto-save 10%"),
            &String::from_str(env, r#"{"percentage":10}"#),
            &String::from_str(env, r#"{"dest":"G..."}"#),
        )
    }

    // ─────────────────────────────────────────────────────────────────────────
    // create_rule
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_rule_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        assert_eq!(id, 1);
        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.owner, owner);
        assert_eq!(rule.rule_type, RuleType::AutoSave);
        assert_eq!(rule.trigger, RuleTrigger::OnIncomingPayment);
        assert_eq!(rule.status, RuleStatus::Active);
        assert_eq!(rule.label, String::from_str(&env, "Auto-save 10%"));
    }

    #[test]
    fn test_create_rule_all_types() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);

        let types_and_triggers = [
            (RuleType::AutoSave, RuleTrigger::OnIncomingPayment),
            (RuleType::AutoSweep, RuleTrigger::OnBalanceAbove),
            (RuleType::Payroll, RuleTrigger::OnSchedule),
            (RuleType::Budget, RuleTrigger::OnCategorySpend),
            (RuleType::Alert, RuleTrigger::OnBalanceBelow),
        ];

        for (i, (rule_type, trigger)) in types_and_triggers.iter().enumerate() {
            let id = client.create_rule(
                &owner,
                rule_type,
                trigger,
                &String::from_str(&env, "Test rule"),
                &String::from_str(&env, "{}"),
                &String::from_str(&env, "{}"),
            );
            assert_eq!(id, (i + 1) as u64);
        }
    }

    #[test]
    fn test_create_rule_auto_increment_id() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = create_sample_rule(&client, &env, &owner);
        let id2 = create_sample_rule(&client, &env, &owner);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_create_rule_updates_wallet_index() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let _id1 = create_sample_rule(&client, &env, &owner);
        let _id2 = create_sample_rule(&client, &env, &owner);

        let rule_ids = client.list_wallet_rules(&owner);
        assert_eq!(rule_ids.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // update_rule
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_update_rule_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.update_rule(
            &owner,
            &id,
            &Some(String::from_str(&env, "Updated label")),
            &Some(String::from_str(&env, r#"{"min_amount":500}"#)),
            &Some(String::from_str(&env, r#"{"percentage":20}"#)),
        );

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.label, String::from_str(&env, "Updated label"));
        assert_eq!(
            rule.trigger_params,
            String::from_str(&env, r#"{"min_amount":500}"#)
        );
        assert_eq!(
            rule.action_params,
            String::from_str(&env, r#"{"percentage":20}"#)
        );
    }

    #[test]
    fn test_update_rule_not_found_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let result = client.try_update_rule(
            &owner,
            &999,
            &Some(String::from_str(&env, "Updated")),
            &None,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::RuleNotFound);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // pause_rule / resume_rule
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_pause_rule_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.pause_rule(&owner, &id);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Paused);
    }

    #[test]
    fn test_resume_rule_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.pause_rule(&owner, &id);
        client.resume_rule(&owner, &id);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Active);
    }

    #[test]
    fn test_pause_deleted_rule_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.delete_rule(&owner, &id);

        let result = client.try_pause_rule(&owner, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyDeleted);
    }

    #[test]
    fn test_resume_deleted_rule_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.delete_rule(&owner, &id);

        let result = client.try_resume_rule(&owner, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyDeleted);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // delete_rule
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_delete_rule_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.delete_rule(&owner, &id);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Deleted);
    }

    #[test]
    fn test_delete_already_deleted_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.delete_rule(&owner, &id);

        let result = client.try_delete_rule(&owner, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyDeleted);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // record_execution
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_record_execution_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.record_execution(&Address::generate(&env), &id);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.execution_count, 1);
    }

    #[test]
    fn test_record_execution_increments_count() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        client.record_execution(&Address::generate(&env), &id);
        client.record_execution(&Address::generate(&env), &id);
        client.record_execution(&Address::generate(&env), &id);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.execution_count, 3);
    }

    #[test]
    fn test_record_execution_not_found_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let result = client.try_record_execution(&Address::generate(&env), &999);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::RuleNotFound);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_rule / list_wallet_rules / list_active_rules
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_rule_returns_correct_record() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.id, id);
        assert_eq!(rule.owner, owner);
        assert_eq!(rule.execution_count, 0);
    }

    #[test]
    fn test_get_rule_missing_returns_none() {
        let (env, client) = setup();

        let result = client.get_rule(&999);
        assert!(result.is_none());
    }

    #[test]
    fn test_list_wallet_rules_returns_all_ids() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = create_sample_rule(&client, &env, &owner);
        let id2 = create_sample_rule(&client, &env, &owner);

        let ids = client.list_wallet_rules(&owner);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids.get_unchecked(0), id1);
        assert_eq!(ids.get_unchecked(1), id2);
    }

    #[test]
    fn test_list_active_rules_filters_correctly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = create_sample_rule(&client, &env, &owner);
        let _id2 = create_sample_rule(&client, &env, &owner);

        client.pause_rule(&owner, &id1);

        let active = client.list_active_rules(&owner);
        assert_eq!(active.len(), 1);
        assert_eq!(active.get_unchecked(0).id, _id2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // lifecycle: create → pause → resume → delete
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_full_rule_lifecycle() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = create_sample_rule(&client, &env, &owner);

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Active);

        client.pause_rule(&owner, &id);
        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Paused);

        client.resume_rule(&owner, &id);
        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Active);

        client.record_execution(&Address::generate(&env), &id);
        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.execution_count, 1);

        client.delete_rule(&owner, &id);
        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.status, RuleStatus::Deleted);
    }

    #[test]
    fn test_list_active_rules_excludes_deleted() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = create_sample_rule(&client, &env, &owner);
        let _id2 = create_sample_rule(&client, &env, &owner);

        client.delete_rule(&owner, &id1);

        let active = client.list_active_rules(&owner);
        assert_eq!(active.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // upgrade functions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        client.initialize(&admin);

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_version(), 1);
        assert!(client.get_wasm_hash().is_some());
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        client.initialize(&admin);

        let result = client.try_initialize(&admin);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    #[test]
    fn test_upgrade_success() {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        client.initialize(&admin);

        let new_hash = Bytes::from_array(&env, &[1u8; 32]);
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

        let hash1 = Bytes::from_array(&env, &[1u8; 32]);
        client.upgrade(&admin, &hash1);
        assert_eq!(client.get_version(), 2);

        let hash2 = Bytes::from_array(&env, &[2u8; 32]);
        client.upgrade(&admin, &hash2);
        assert_eq!(client.get_version(), 3);
    }

    #[test]
    fn test_upgrade_same_hash_fails() {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        client.initialize(&admin);

        let hash = Bytes::from_array(&env, &[0u8; 32]);
        let result = client.try_upgrade(&admin, &hash);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::SameWasmHash);
    }

    #[test]
    fn test_upgrade_unauthorized_fails() {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);

        client.initialize(&admin);

        let new_hash = Bytes::from_array(&env, &[1u8; 32]);
        let result = client.try_upgrade(&attacker, &new_hash);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_upgrade_before_init_fails() {
        let (env, client) = setup();
        env.mock_all_auths();
        let caller = Address::generate(&env);

        let new_hash = Bytes::from_array(&env, &[1u8; 32]);
        let result = client.try_upgrade(&caller, &new_hash);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotInitialized);
    }

    #[test]
    fn test_get_admin_before_init_returns_none() {
        let (env, client) = setup();
        assert_eq!(client.get_admin(), None);
    }

    #[test]
    fn test_get_version_before_init_returns_initial() {
        let (env, client) = setup();
        assert_eq!(client.get_version(), 1);
    }
}
