#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    use crate::contract::MultiSigContract;
    use crate::contract::MultiSigContractClient;
    use crate::errors::ContractError;

    fn setup<'a>() -> (Env, MultiSigContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, MultiSigContract);
        let client = MultiSigContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, MultiSigContractClient<'a>, Address, Vec<Address>) {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signer3 = Address::generate(&env);

        let mut signers = Vec::new(&env);
        signers.push_back(signer1.clone());
        signers.push_back(signer2.clone());
        signers.push_back(signer3.clone());

        client.initialize(&admin, &signers, &2);

        (env, client, admin, signers)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // initialization
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let (env, client, admin, signers) = setup_initialized();

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_threshold(), Some(2));

        let stored_signers = client.get_signers();
        assert_eq!(stored_signers.len(), 3);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let mut signers = Vec::new(&env);
        signers.push_back(signer1);

        client.initialize(&admin, &signers, &1);

        let result = client.try_initialize(&admin, &signers, &1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    #[test]
    fn test_initialize_invalid_threshold_zero() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let mut signers = Vec::new(&env);
        signers.push_back(signer1);

        let result = client.try_initialize(&admin, &signers, &0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidThreshold
        );
    }

    #[test]
    fn test_initialize_threshold_exceeds_signers() {
        let (env, client) = setup();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer1 = Address::generate(&env);
        let mut signers = Vec::new(&env);
        signers.push_back(signer1);

        let result = client.try_initialize(&admin, &signers, &3);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidThreshold
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // signer management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_add_signer_success() {
        let (env, client, admin, _) = setup_initialized();
        let new_signer = Address::generate(&env);

        client.add_signer(&admin, &new_signer);

        let signers = client.get_signers();
        assert_eq!(signers.len(), 4);
    }

    #[test]
    fn test_add_signer_unauthorized_fails() {
        let (env, client, _, signers) = setup_initialized();
        let attacker = Address::generate(&env);
        let new_signer = Address::generate(&env);

        let result = client.try_add_signer(&attacker, &new_signer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_add_signer_duplicate_fails() {
        let (env, client, admin, signers) = setup_initialized();

        let result = client.try_add_signer(&admin, &signers.get_unchecked(0));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::SignerAlreadyExists
        );
    }

    #[test]
    fn test_remove_signer_success() {
        let (env, client, admin, signers) = setup_initialized();

        client.remove_signer(&admin, &signers.get_unchecked(2));

        let stored_signers = client.get_signers();
        assert_eq!(stored_signers.len(), 3);
        let active_count = stored_signers.iter().filter(|s| s.active).count();
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_remove_signer_not_found_fails() {
        let (env, client, admin, _) = setup_initialized();
        let unknown = Address::generate(&env);

        let result = client.try_remove_signer(&admin, &unknown);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::SignerNotFound);
    }

    #[test]
    fn test_remove_signer_cannot_remove_self_fails() {
        let (env, client, admin, _) = setup_initialized();

        let result = client.try_remove_signer(&admin, &admin);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::CannotRemoveSelf
        );
    }

    #[test]
    fn test_remove_signer_threshold_exceeds_fails() {
        let (env, client, admin, signers) = setup_initialized();

        client.remove_signer(&admin, &signers.get_unchecked(2));

        let result = client.try_remove_signer(&admin, &signers.get_unchecked(1));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::ThresholdExceedsSigners
        );
    }

    #[test]
    fn test_update_threshold_success() {
        let (env, client, admin, _) = setup_initialized();

        client.update_threshold(&admin, &3);
        assert_eq!(client.get_threshold(), Some(3));
    }

    #[test]
    fn test_update_threshold_zero_fails() {
        let (env, client, admin, _) = setup_initialized();

        let result = client.try_update_threshold(&admin, &0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidThreshold
        );
    }

    #[test]
    fn test_update_threshold_exceeds_signers_fails() {
        let (env, client, admin, _) = setup_initialized();

        let result = client.try_update_threshold(&admin, &10);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::ThresholdExceedsSigners
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // proposal management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_proposal_success() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(id, 1);
        assert_eq!(client.total_proposals(), 1);

        let proposal = client.get_proposal(&id).unwrap();
        assert_eq!(proposal.proposer, signers.get_unchecked(0));
        assert!(!proposal.executed);
        assert!(!proposal.cancelled);
    }

    #[test]
    fn test_create_proposal_non_signer_fails() {
        let (env, client, _, _) = setup_initialized();
        let non_signer = Address::generate(&env);
        let target = Address::generate(&env);

        let result = client.try_create_proposal(
            &non_signer,
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::Unauthorized);
    }

    #[test]
    fn test_sign_proposal_success() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.sign_proposal(&signers.get_unchecked(0), &id);
        client.sign_proposal(&signers.get_unchecked(1), &id);

        let signed_by = client.get_proposal_signers(&id);
        assert_eq!(signed_by.len(), 2);
        assert!(client.has_signed(&signers.get_unchecked(0), &id));
        assert!(client.has_signed(&signers.get_unchecked(1), &id));
    }

    #[test]
    fn test_sign_proposal_already_signed_fails() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.sign_proposal(&signers.get_unchecked(0), &id);

        let result = client.try_sign_proposal(&signers.get_unchecked(0), &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::AlreadySigned);
    }

    #[test]
    fn test_execute_proposal_success() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.sign_proposal(&signers.get_unchecked(0), &id);
        client.sign_proposal(&signers.get_unchecked(1), &id);

        client.execute_proposal(&signers.get_unchecked(2), &id);

        let proposal = client.get_proposal(&id).unwrap();
        assert!(proposal.executed);
    }

    #[test]
    fn test_execute_proposal_not_enough_signatures_fails() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.sign_proposal(&signers.get_unchecked(0), &id);

        let result = client.try_execute_proposal(&signers.get_unchecked(1), &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::NotReadyToExecute
        );
    }

    #[test]
    fn test_execute_proposal_already_executed_fails() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.sign_proposal(&signers.get_unchecked(0), &id);
        client.sign_proposal(&signers.get_unchecked(1), &id);
        client.execute_proposal(&signers.get_unchecked(2), &id);

        let result = client.try_execute_proposal(&signers.get_unchecked(0), &id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyExecuted
        );
    }

    #[test]
    fn test_cancel_proposal_success() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.cancel_proposal(&signers.get_unchecked(0), &id);

        let proposal = client.get_proposal(&id).unwrap();
        assert!(proposal.cancelled);
    }

    #[test]
    fn test_cancel_proposal_by_admin_success() {
        let (env, client, admin, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.cancel_proposal(&admin, &id);

        let proposal = client.get_proposal(&id).unwrap();
        assert!(proposal.cancelled);
    }

    #[test]
    fn test_cancel_proposal_unauthorized_fails() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        let id = client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        let other = Address::generate(&env);
        let result = client.try_cancel_proposal(&other, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::Unauthorized);
    }

    #[test]
    fn test_get_proposal_not_found() {
        let (env, client) = setup();
        assert!(client.get_proposal(&999).is_none());
    }

    #[test]
    fn test_total_proposals() {
        let (env, client, _, signers) = setup_initialized();
        let target = Address::generate(&env);

        assert_eq!(client.total_proposals(), 0);

        client.create_proposal(
            &signers.get_unchecked(0),
            &target,
            &String::from_str(&env, "upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(client.total_proposals(), 1);
    }
}
