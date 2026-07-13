#[cfg(test)]
mod bench {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

    use crate::contract::AutomationRulesContract;
    use crate::contract::AutomationRulesContractClient;
    use crate::types::{RuleTrigger, RuleType};

    fn setup<'a>() -> (Env, AutomationRulesContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, AutomationRulesContract);
        let client = AutomationRulesContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn bench_create_rule() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let start = env.ledger().sequence();

        for _ in 0..100 {
            client.create_rule(
                &owner,
                &RuleType::AutoSave,
                &RuleTrigger::OnIncomingPayment,
                &String::from_str(&env, "Rule"),
                &String::from_str(&env, "{}"),
                &String::from_str(&env, "{}"),
            );
        }

        let end = env.ledger().sequence();
        let elapsed = end - start;
        assert!(
            elapsed < 100,
            "100 rules created in {} ledgers (expected < 100)",
            elapsed
        );
    }

    #[test]
    fn bench_list_wallet_rules() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        for _ in 0..50 {
            client.create_rule(
                &owner,
                &RuleType::AutoSave,
                &RuleTrigger::OnIncomingPayment,
                &String::from_str(&env, "Rule"),
                &String::from_str(&env, "{}"),
                &String::from_str(&env, "{}"),
            );
        }

        let start = env.ledger().sequence();
        let _ids = client.list_wallet_rules(&owner);
        let end = env.ledger().sequence();

        let elapsed = end - start;
        assert!(
            elapsed < 10,
            "List 50 rules took {} ledgers (expected < 10)",
            elapsed
        );
    }

    #[test]
    fn bench_record_execution() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_rule(
            &owner,
            &RuleType::AutoSave,
            &RuleTrigger::OnIncomingPayment,
            &String::from_str(&env, "Rule"),
            &String::from_str(&env, "{}"),
            &String::from_str(&env, "{}"),
        );

        let caller = Address::generate(&env);
        let start = env.ledger().sequence();

        for _ in 0..100 {
            client.record_execution(&caller, &id);
        }

        let end = env.ledger().sequence();
        let elapsed = end - start;
        assert!(
            elapsed < 100,
            "100 executions took {} ledgers (expected < 100)",
            elapsed
        );

        let rule = client.get_rule(&id).unwrap();
        assert_eq!(rule.execution_count, 100);
    }
}
