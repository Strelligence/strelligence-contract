#[cfg(test)]
mod test {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Bytes, Env, String};

    use crate::contract::SnapshotsContract;
    use crate::contract::SnapshotsContractClient;
    use crate::errors::ContractError;

    fn setup<'a>() -> (Env, SnapshotsContractClient<'a>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, SnapshotsContract);
        let client = SnapshotsContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn setup_initialized<'a>() -> (Env, SnapshotsContractClient<'a>, Address) {
        let (env, client) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // initialization
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_initialize_success() {
        let (env, client, admin) = setup_initialized();

        assert_eq!(client.get_admin(), Some(admin));
        assert_eq!(client.get_version(), 1);
    }

    #[test]
    fn test_initialize_twice_fails() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_initialize(&admin);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // snapshot management
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_create_snapshot_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "pre-upgrade"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(id, 1);
        assert_eq!(client.total_snapshots(), 1);

        let snapshot = client.get_snapshot(&id).unwrap();
        assert_eq!(snapshot.creator, admin);
        assert_eq!(snapshot.contract_address, contract_addr);
        assert_eq!(snapshot.label, String::from_str(&env, "pre-upgrade"));
        assert!(!snapshot.expired);
    }

    #[test]
    fn test_create_snapshot_unauthorized_fails() {
        let (env, client, _) = setup_initialized();
        let non_admin = Address::generate(&env);
        let contract_addr = Address::generate(&env);

        let result = client.try_create_snapshot(
            &non_admin,
            &contract_addr,
            &String::from_str(&env, "test"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    #[test]
    fn test_create_snapshot_empty_label_fails() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let result = client.try_create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, ""),
            &Bytes::from_array(&env, &[0u8; 32]),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::InvalidLabel
        );
    }

    #[test]
    fn test_create_multiple_snapshots() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id1 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snapshot-1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        let id2 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snapshot-2"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(client.total_snapshots(), 2);
    }

    #[test]
    fn test_delete_snapshot_success() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "to-delete"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.delete_snapshot(&admin, &id);

        let snapshot = client.get_snapshot(&id).unwrap();
        assert!(snapshot.expired);
    }

    #[test]
    fn test_delete_snapshot_not_found_fails() {
        let (env, client, admin) = setup_initialized();

        let result = client.try_delete_snapshot(&admin, &999);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::SnapshotNotFound
        );
    }

    #[test]
    fn test_delete_snapshot_unauthorized_fails() {
        let (env, client, admin) = setup_initialized();
        let non_admin = Address::generate(&env);
        let contract_addr = Address::generate(&env);

        let id = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "test"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        let result = client.try_delete_snapshot(&non_admin, &id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::NotAdmin);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // snapshot comparison
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compare_snapshots_identical() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let state = Bytes::from_array(&env, &[42u8; 32]);

        let id1 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-1"),
            &state,
        );

        let id2 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-2"),
            &state,
        );

        let diff = client.compare_snapshots(&id1, &id2);
        assert!(diff.identical);
        assert_eq!(diff.modified_keys, 0);
    }

    #[test]
    fn test_compare_snapshots_different() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id1 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        let id2 = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-2"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        let diff = client.compare_snapshots(&id1, &id2);
        assert!(!diff.identical);
        assert_eq!(diff.modified_keys, 1);
    }

    #[test]
    fn test_compare_snapshots_not_found_fails() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        let result = client.try_compare_snapshots(&id, &999);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::SnapshotNotFound
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // read functions
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_get_contract_snapshots() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-1"),
            &Bytes::from_array(&env, &[1u8; 32]),
        );

        client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-2"),
            &Bytes::from_array(&env, &[2u8; 32]),
        );

        let snapshots = client.get_contract_snapshots(&contract_addr);
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_get_contract_snapshots_excludes_expired() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        let id = client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        client.delete_snapshot(&admin, &id);

        let snapshots = client.get_contract_snapshots(&contract_addr);
        assert_eq!(snapshots.len(), 0);
    }

    #[test]
    fn test_get_snapshot_not_found() {
        let (env, client) = setup();
        assert!(client.get_snapshot(&999).is_none());
    }

    #[test]
    fn test_total_snapshots() {
        let (env, client, admin) = setup_initialized();
        let contract_addr = Address::generate(&env);

        assert_eq!(client.total_snapshots(), 0);

        client.create_snapshot(
            &admin,
            &contract_addr,
            &String::from_str(&env, "snap-1"),
            &Bytes::from_array(&env, &[0u8; 32]),
        );

        assert_eq!(client.total_snapshots(), 1);
    }
}
