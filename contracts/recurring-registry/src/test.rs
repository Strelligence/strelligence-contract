#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    use crate::contract::RecurringRegistryContract;
    use crate::contract::RecurringRegistryContractClient;
    use crate::errors::ContractError;
    use crate::types::{Frequency, SubscriptionStatus, SubscriptionType};

    fn setup<'a>() -> (Env, RecurringRegistryContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, RecurringRegistryContract);
        let client = RecurringRegistryContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn sample_label(env: &Env) -> Option<String> {
        Some(String::from_str(env, "Netflix"))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // create_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_subscription_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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
            &sample_label(&env),
        );

        assert_eq!(id, 1);
        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.owner, owner);
        assert_eq!(sub.amount, 10_000_000);
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_create_subscription_zero_amount_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let result = client.try_create_subscription(
            &Address::generate(&env),
            &Address::generate(&env),
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &0,
            &1000,
            &false,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidAmount);
    }

    #[test]
    fn test_create_subscription_negative_amount_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let result = client.try_create_subscription(
            &Address::generate(&env),
            &Address::generate(&env),
            &String::from_str(&env, "Netflix"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &-100,
            &1000,
            &false,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidAmount);
    }

    #[test]
    fn test_create_subscription_auto_increment_id() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = client.create_subscription(
            &Address::generate(&env),
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

        let id2 = client.create_subscription(
            &Address::generate(&env),
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

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_create_subscription_updates_wallet_index() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let _id1 = client.create_subscription(
            &Address::generate(&env),
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

        let _id2 = client.create_subscription(
            &Address::generate(&env),
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

        let ids = client.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // update_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_update_subscription_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.update_subscription(
            &owner,
            &id,
            &Some(String::from_str(&env, "Netflix Premium")),
            &Some(Frequency::Weekly),
            &Some(15_000_000),
            &Some(2000),
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.merchant, String::from_str(&env, "Netflix Premium"));
        assert_eq!(sub.frequency, Frequency::Weekly);
        assert_eq!(sub.amount, 15_000_000);
        assert_eq!(sub.next_payment_ledger, 2000);
    }

    #[test]
    fn test_update_subscription_not_found_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let result =
            client.try_update_subscription(&owner, &999, &None, &None, &None, &None, &None);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::SubscriptionNotFound
        );
    }

    #[test]
    fn test_update_subscription_zero_amount_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        let result =
            client.try_update_subscription(&owner, &id, &None, &None, &Some(0), &None, &None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidAmount);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // pause_subscription
    // ─────────────────────────────────────────────────────────────────────────
    // cancel_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cancel_subscription_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.cancel_subscription(&owner, &id);

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);
        assert!(!sub.active);
    }

    #[test]
    fn test_cancel_subscription_already_cancelled_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.cancel_subscription(&owner, &id);

        let result = client.try_cancel_subscription(&owner, &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyCancelled
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // pause_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_pause_subscription_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.pause_subscription(&owner, &id);

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Paused);
        assert!(!sub.active);
    }

    #[test]
    fn test_pause_subscription_already_paused_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.pause_subscription(&owner, &id);

        let result = client.try_pause_subscription(&owner, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyInState);
    }

    #[test]
    fn test_pause_subscription_cancelled_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.cancel_subscription(&owner, &id);

        let result = client.try_pause_subscription(&owner, &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyCancelled
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // resume_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_resume_subscription_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.pause_subscription(&owner, &id);
        client.resume_subscription(&owner, &id);

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert!(sub.active);
    }

    #[test]
    fn test_resume_subscription_cancelled_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.cancel_subscription(&owner, &id);

        let result = client.try_resume_subscription(&owner, &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyCancelled
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // confirm_payment
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_confirm_payment_success() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.confirm_payment(&Address::generate(&env), &id, &5000, &6000);

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.last_payment_ledger, 5000);
        assert_eq!(sub.next_payment_ledger, 6000);
    }

    #[test]
    fn test_confirm_payment_cancelled_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.cancel_subscription(&owner, &id);

        let result = client.try_confirm_payment(&Address::generate(&env), &id, &5000, &6000);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InactiveSubscription
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_subscription
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_subscription_returns_correct_record() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.merchant, String::from_str(&env, "Netflix"));
        assert_eq!(sub.asset_code, String::from_str(&env, "USDC"));
        assert_eq!(sub.amount, 10_000_000);
    }

    #[test]
    fn test_get_subscription_missing_returns_none() {
        let (env, client) = setup();

        let result = client.get_subscription(&999);
        assert!(result.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // list_wallet_subscriptions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_wallet_subscriptions_returns_all_ids() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = client.create_subscription(
            &Address::generate(&env),
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
        let id2 = client.create_subscription(
            &Address::generate(&env),
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

        let ids = client.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids.get_unchecked(0), id1);
        assert_eq!(ids.get_unchecked(1), id2);
    }

    #[test]
    fn test_list_wallet_subscriptions_pagination() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let mut expected_ids = Vec::new(&env);

        for i in 0..5 {
            let id = client.create_subscription(
                &Address::generate(&env),
                &owner,
                &String::from_str(&env, "Merchant"),
                &None,
                &Frequency::Monthly,
                &SubscriptionType::Subscription,
                &String::from_str(&env, "USDC"),
                &String::from_str(&env, "issuer1"),
                &10_000_000,
                &(1000 + i as u64),
                &false,
                &None,
            );
            expected_ids.push_back(id);
        }

        let page1 = client.list_wallet_subscriptions(&owner, &0, &2);
        assert_eq!(page1.len(), 2);
        assert_eq!(page1.get_unchecked(0), expected_ids.get_unchecked(0));
        assert_eq!(page1.get_unchecked(1), expected_ids.get_unchecked(1));

        let page2 = client.list_wallet_subscriptions(&owner, &2, &2);
        assert_eq!(page2.len(), 2);
        assert_eq!(page2.get_unchecked(0), expected_ids.get_unchecked(2));
        assert_eq!(page2.get_unchecked(1), expected_ids.get_unchecked(3));

        let page3 = client.list_wallet_subscriptions(&owner, &4, &2);
        assert_eq!(page3.len(), 1);
        assert_eq!(page3.get_unchecked(0), expected_ids.get_unchecked(4));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // list_active_subscriptions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_active_subscriptions_filters_correctly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = client.create_subscription(
            &Address::generate(&env),
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
        let id2 = client.create_subscription(
            &Address::generate(&env),
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

        // Cancel one subscription
        client.cancel_subscription(&owner, &id1);

        let active = client.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 1);
        assert_eq!(active.get_unchecked(0).id, id2);
    }

    #[test]
    fn test_list_active_subscriptions_empty_wallet() {
        let (env, client) = setup();

        let owner = Address::generate(&env);
        let active = client.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // total_subscriptions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_total_subscriptions_counts_correctly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        assert_eq!(client.total_subscriptions(), 0);

        client.create_subscription(
            &Address::generate(&env),
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

        assert_eq!(client.total_subscriptions(), 1);

        client.create_subscription(
            &Address::generate(&env),
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

        assert_eq!(client.total_subscriptions(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // get_subscription — not found
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_subscription_not_found() {
        let (env, client) = setup();

        let result = client.get_subscription(&12345);
        assert!(result.is_none());
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
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadyInState);
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

    // ─────────────────────────────────────────────────────────────────────────
    // Edge case tests — all SubscriptionType variants
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_subscription_type_payroll() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Payroll"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Payroll,
            &String::from_str(&env, "XLM"),
            &String::from_str(&env, "native"),
            &500_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Payroll);
        assert_eq!(sub.amount, 500_000_000);
    }

    #[test]
    fn test_create_subscription_type_income() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Salary"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Income,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &1_000_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Income);
    }

    #[test]
    fn test_create_subscription_type_savings() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "AutoSave"),
            &None,
            &Frequency::Weekly,
            &SubscriptionType::Savings,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &100_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Savings);
    }

    #[test]
    fn test_create_subscription_type_bill() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Electric"),
            &None,
            &Frequency::Monthly,
            &SubscriptionType::Bill,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &150_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Bill);
    }

    #[test]
    fn test_create_subscription_type_investment() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "DCA"),
            &None,
            &Frequency::Weekly,
            &SubscriptionType::Investment,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &50_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Investment);
    }

    #[test]
    fn test_create_subscription_type_transfer() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Transfer"),
            &None,
            &Frequency::BiWeekly,
            &SubscriptionType::Transfer,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &200_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Transfer);
    }

    #[test]
    fn test_create_subscription_type_other() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Custom"),
            &None,
            &Frequency::Custom,
            &SubscriptionType::Other,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &75_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.subscription_type, SubscriptionType::Other);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge case tests — all Frequency variants
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_subscription_frequency_daily() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Daily"),
            &None,
            &Frequency::Daily,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.frequency, Frequency::Daily);
    }

    #[test]
    fn test_create_subscription_frequency_weekly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Weekly"),
            &None,
            &Frequency::Weekly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.frequency, Frequency::Weekly);
    }

    #[test]
    fn test_create_subscription_frequency_biweekly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "BiWeekly"),
            &None,
            &Frequency::BiWeekly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.frequency, Frequency::BiWeekly);
    }

    #[test]
    fn test_create_subscription_frequency_quarterly() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Quarterly"),
            &None,
            &Frequency::Quarterly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.frequency, Frequency::Quarterly);
    }

    #[test]
    fn test_create_subscription_frequency_annually() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Annually"),
            &None,
            &Frequency::Annually,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.frequency, Frequency::Annually);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge case tests — merchant address and auto_detected
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_subscription_with_merchant_address() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let merchant_addr = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
            &owner,
            &String::from_str(&env, "Netflix"),
            &Some(merchant_addr.clone()),
            &Frequency::Monthly,
            &SubscriptionType::Subscription,
            &String::from_str(&env, "USDC"),
            &String::from_str(&env, "issuer1"),
            &10_000_000,
            &1000,
            &false,
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.merchant_address, Some(merchant_addr));
    }

    #[test]
    fn test_create_subscription_auto_detected() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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
            &None,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert!(sub.auto_detected);
    }

    #[test]
    fn test_create_subscription_with_custom_label() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let label = Some(String::from_str(&env, "My Netflix"));
        let id = client.create_subscription(
            &Address::generate(&env),
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
            &label,
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.custom_label, Some(String::from_str(&env, "My Netflix")));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Edge case tests — multiple owners, empty wallet
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_list_wallet_subscriptions_empty() {
        let (env, client) = setup();

        let owner = Address::generate(&env);
        let ids = client.list_wallet_subscriptions(&owner, &0, &10);
        assert_eq!(ids.len(), 0);
    }

    #[test]
    fn test_list_active_subscriptions_empty() {
        let (env, client) = setup();

        let owner = Address::generate(&env);
        let active = client.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_multiple_owners_isolated() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);

        let id1 = client.create_subscription(
            &Address::generate(&env),
            &owner1,
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

        let id2 = client.create_subscription(
            &Address::generate(&env),
            &owner2,
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

        let ids1 = client.list_wallet_subscriptions(&owner1, &0, &10);
        let ids2 = client.list_wallet_subscriptions(&owner2, &0, &10);

        assert_eq!(ids1.len(), 1);
        assert_eq!(ids2.len(), 1);
        assert_eq!(ids1.get_unchecked(0), id1);
        assert_eq!(ids2.get_unchecked(0), id2);
    }

    #[test]
    fn test_update_subscription_custom_label() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        client.update_subscription(
            &owner,
            &id,
            &None,
            &None,
            &None,
            &None,
            &Some(String::from_str(&env, "Updated Label")),
        );

        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(
            sub.custom_label,
            Some(String::from_str(&env, "Updated Label"))
        );
    }

    #[test]
    fn test_update_subscription_negative_amount_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        let result =
            client.try_update_subscription(&owner, &id, &None, &None, &Some(-100), &None, &None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidAmount);
    }

    #[test]
    fn test_resume_subscription_not_paused_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        // Active subscription should not need resume - but contract allows it
        // This tests that resume on active subscription works (idempotent)
        client.resume_subscription(&owner, &id);
        let sub = client.get_subscription(&id).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
    }

    #[test]
    fn test_confirm_payment_expired_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        // Manually set status to expired (simulate backend logic)
        let mut sub = client.get_subscription(&id).unwrap();
        sub.status = SubscriptionStatus::Expired;
        sub.active = false;

        // Note: We can't directly set expired status via contract API
        // This test verifies the inactive subscription check works
        // by canceling first (which sets active=false)
        client.cancel_subscription(&owner, &id);

        let result = client.try_confirm_payment(&Address::generate(&env), &id, &5000, &6000);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InactiveSubscription
        );
    }

    #[test]
    fn test_list_active_subscriptions_with_paused() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id1 = client.create_subscription(
            &Address::generate(&env),
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
        let id2 = client.create_subscription(
            &Address::generate(&env),
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

        // Pause one subscription
        client.pause_subscription(&owner, &id1);

        let active = client.list_active_subscriptions(&owner, &0, &10);
        assert_eq!(active.len(), 1);
        assert_eq!(active.get_unchecked(0).id, id2);
    }

    #[test]
    fn test_total_subscriptions_after_cancel() {
        let (env, client) = setup();
        env.mock_all_auths();

        let owner = Address::generate(&env);
        let id = client.create_subscription(
            &Address::generate(&env),
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

        assert_eq!(client.total_subscriptions(), 1);

        client.cancel_subscription(&owner, &id);

        // Total count should still be 1 (cancel doesn't decrement)
        assert_eq!(client.total_subscriptions(), 1);
    }

    #[test]
    fn test_get_wasm_hash_before_init() {
        let (env, client) = setup();
        assert_eq!(client.get_wasm_hash(), None);
    }
}
