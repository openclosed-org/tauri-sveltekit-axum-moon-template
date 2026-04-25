//! MockAuthzAdapter unit tests.

#[cfg(test)]
mod tests {
    use crate::{AuthzPort, AuthzTupleKey, MockAuthzAdapter};

    #[tokio::test]
    async fn allow_all_when_empty() {
        let authz = MockAuthzAdapter::new();
        assert!(authz.is_empty().await);
        // When store is empty, all checks return true (dev convenience)
        assert!(
            authz
                .check("user:alice", "owner", "tenant:acme")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn strict_mode_denies_unknown() {
        let authz = MockAuthzAdapter::strict();
        // Strict mode: empty store returns false
        assert!(
            !authz
                .check("user:alice", "owner", "tenant:acme")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn write_and_check_tuple() {
        let authz = MockAuthzAdapter::strict();
        authz
            .write_tuple(&AuthzTupleKey::new("user:alice", "owner", "tenant:acme"))
            .await
            .unwrap();

        assert!(
            authz
                .check("user:alice", "owner", "tenant:acme")
                .await
                .unwrap()
        );
        assert!(
            !authz
                .check("user:bob", "owner", "tenant:acme")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn delete_tuple() {
        let authz = MockAuthzAdapter::strict();
        let key = AuthzTupleKey::new("user:alice", "owner", "tenant:acme");
        authz.write_tuple(&key).await.unwrap();
        assert!(
            authz
                .check("user:alice", "owner", "tenant:acme")
                .await
                .unwrap()
        );

        authz.delete_tuple(&key).await.unwrap();
        assert!(
            !authz
                .check("user:alice", "owner", "tenant:acme")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn list_tuples_filter() {
        let authz = MockAuthzAdapter::strict();
        authz
            .write_tuple(&AuthzTupleKey::new("user:alice", "owner", "tenant:acme"))
            .await
            .unwrap();
        authz
            .write_tuple(&AuthzTupleKey::new("user:alice", "member", "tenant:beta"))
            .await
            .unwrap();
        authz
            .write_tuple(&AuthzTupleKey::new("user:bob", "member", "tenant:acme"))
            .await
            .unwrap();

        let alice_tuples = authz
            .list_tuples(Some("user:alice"), None, None)
            .await
            .unwrap();
        assert_eq!(alice_tuples.len(), 2);

        let owner_tuples = authz.list_tuples(None, Some("owner"), None).await.unwrap();
        assert_eq!(owner_tuples.len(), 1);
    }

    #[tokio::test]
    async fn batch_check() {
        let authz = MockAuthzAdapter::strict();
        authz
            .write_tuple(&AuthzTupleKey::new(
                "user:alice",
                "can_write",
                "counter:acme",
            ))
            .await
            .unwrap();

        let checks = vec![
            (
                "user:alice".to_string(),
                "can_write".to_string(),
                "counter:acme".to_string(),
            ),
            (
                "user:bob".to_string(),
                "can_write".to_string(),
                "counter:acme".to_string(),
            ),
        ];
        let results = authz.batch_check(&checks).await.unwrap();
        assert_eq!(results, vec![true, false]);
    }

    #[tokio::test]
    async fn seed_works() {
        let authz = MockAuthzAdapter::strict();
        authz
            .seed(vec![
                AuthzTupleKey::new("user:dev", "owner", "tenant:dev-001"),
                AuthzTupleKey::new("user:dev", "can_write", "counter:dev-001"),
            ])
            .await;

        assert_eq!(authz.len().await, 2);
        assert!(
            authz
                .check("user:dev", "owner", "tenant:dev-001")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn seeded_member_and_counter_permissions_can_be_checked() {
        let authz = MockAuthzAdapter::strict();
        authz
            .seed(vec![
                AuthzTupleKey::new("user:bound-user", "owner", "tenant:tenant-123"),
                AuthzTupleKey::new("user:bound-user", "member", "tenant:tenant-123"),
                AuthzTupleKey::new("user:bound-user", "can_read", "counter:tenant-123"),
                AuthzTupleKey::new("user:bound-user", "can_write", "counter:tenant-123"),
            ])
            .await;

        assert!(
            authz
                .check("user:bound-user", "member", "tenant:tenant-123")
                .await
                .unwrap()
        );
        assert!(
            authz
                .check("user:bound-user", "can_read", "counter:tenant-123")
                .await
                .unwrap()
        );
        assert!(
            authz
                .check("user:bound-user", "can_write", "counter:tenant-123")
                .await
                .unwrap()
        );
    }
}
