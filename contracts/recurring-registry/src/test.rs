#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String};

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
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidAmount
        );
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
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidAmount
        );
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

        let ids = client.list_wallet_subscriptions(&owner);
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
        let result = client.try_update_subscription(
            &owner,
            &999,
            &None,
            &None,
            &None,
            &None,
            &None,
        );

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

        let result = client.try_update_subscription(
            &owner,
            &id,
            &None,
            &None,
            &Some(0),
            &None,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidAmount
        );
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
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInState
        );
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

        let ids = client.list_wallet_subscriptions(&owner);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids.get_unchecked(0), id1);
        assert_eq!(ids.get_unchecked(1), id2);
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

        let active = client.list_active_subscriptions(&owner);
        assert_eq!(active.len(), 1);
        assert_eq!(active.get_unchecked(0).id, id2);
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
}
