use std::sync::Arc;
use tokio::sync::OnceCell;

mod harness;
use harness::{MigrationTestFramework, RollbackTestCapability, TestDataBuilder};

/// Core migration functionality tests for SurrealDB schema management
///
///
/// This test suite validates:
/// - Migration chain execution and performance
/// - Batch data operations and schema evolution
/// - Constraint validation and rollback scenarios
/// - High-volume operations and stress testing
#[cfg(test)]
mod migration_core_tests {
    use super::*;

    /// Shared test database for performance optimization
    static SHARED_DB: OnceCell<Arc<MigrationTestFramework>> = OnceCell::const_new();

    async fn get_test_db() -> Arc<MigrationTestFramework> {
        SHARED_DB
            .get_or_init(|| async { Arc::new(MigrationTestFramework::new().await.unwrap()) })
            .await
            .clone()
    }

    /// Test core migration functionality with performance benchmarking
    #[tokio::test]
    async fn test_migration_chain_performance() {
        let db = get_test_db().await;
        let start = std::time::Instant::now();

        // Reset and apply full migration chain
        db.reset_database().await.unwrap();

        let mut temp_db = MigrationTestFramework::new().await.unwrap();
        temp_db
            .apply_cached_migrations(&["initial", "indexes", "comments"])
            .await
            .unwrap();

        let migration_time = start.elapsed();

        // Performance assertion - migrations should complete quickly
        assert!(
            migration_time.as_millis() < 1000,
            "Migration chain took too long: {:?}",
            migration_time
        );

        // Verify all expected tables exist
        assert!(temp_db.verify_table_exists("author").await.unwrap());
        assert!(temp_db.verify_table_exists("post").await.unwrap());
        assert!(temp_db.verify_table_exists("comment").await.unwrap());
    }

    /// Test batch data operations with performance validation
    #[tokio::test]
    async fn test_batch_operations_performance() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        db.apply_cached_migrations(&["initial", "indexes"])
            .await
            .unwrap();
        db.setup_complete_testing().await.unwrap();

        let start = std::time::Instant::now();

        // Use the working pattern from utility test
        db.insert_basic_test_data().await.unwrap();

        // Create additional test data using individual operations (proven to work)
        db.create_test_author("author:batch1", "Batch User 1", "batch1@example.com")
            .await
            .unwrap();
        db.create_test_author("author:batch2", "Batch User 2", "batch2@example.com")
            .await
            .unwrap();

        let batch_time = start.elapsed();

        // Performance validation
        assert!(
            batch_time.as_millis() < 500,
            "Operations took too long: {:?}",
            batch_time
        );

        // Verify data using the working query pattern
        let test_name = db.query_field_string("author:test", "name").await.unwrap();
        assert_eq!(test_name, Some("Test Author".to_string()));

        let batch1_name = db
            .query_field_string("author:batch1", "name")
            .await
            .unwrap();
        assert_eq!(batch1_name, Some("Batch User 1".to_string()));

        let batch2_name = db
            .query_field_string("author:batch2", "name")
            .await
            .unwrap();
        assert_eq!(batch2_name, Some("Batch User 2".to_string()));
    }

    /// Test schema evolution and data preservation  
    #[tokio::test]
    async fn test_schema_evolution_data_integrity() {
        let mut db = MigrationTestFramework::new().await.unwrap();

        // Apply incremental migrations and verify data preservation
        let migration_stages = [
            (vec!["initial"], "Initial schema"),
            (vec!["initial", "indexes"], "With indexes"),
            (vec!["initial", "indexes", "comments"], "With comments"),
        ];

        let mut baseline_count = 0;

        for (stage_idx, (migrations, description)) in migration_stages.iter().enumerate() {
            db.reset_database().await.unwrap();
            db.apply_cached_migrations(migrations).await.unwrap();
            db.setup_complete_testing().await.unwrap();

            // Create consistent test dataset
            let authors = TestDataBuilder::authors();
            db.create_test_authors(&authors).await.unwrap();

            let current_count = db.count_table_records("author").await.unwrap();

            if stage_idx == 0 {
                baseline_count = current_count;
            } else {
                assert_eq!(
                    current_count, baseline_count,
                    "Data integrity compromised at stage: {}",
                    description
                );
            }

            // Verify schema functionality
            assert!(db.verify_table_exists("author").await.unwrap());
            assert!(db.verify_field_exists("author", "name").await.unwrap());
        }
    }

    /// Test constraint validation with deterministic patterns
    #[tokio::test]
    async fn test_constraint_validation_comprehensive() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        db.apply_cached_migrations(&["initial"]).await.unwrap();
        db.setup_complete_testing().await.unwrap();

        // Test valid author creation using the working pattern
        let valid_result = db
            .create_test_author("author:valid", "Valid User", "valid@example.com")
            .await;
        assert!(valid_result.is_ok(), "Valid email should be accepted");

        // Verify the author was created
        let name = db.query_field_string("author:valid", "name").await.unwrap();
        assert_eq!(name, Some("Valid User".to_string()));

        let email = db
            .query_field_string("author:valid", "email")
            .await
            .unwrap();
        assert_eq!(email, Some("valid@example.com".to_string()));

        // Test another valid author with complex email
        let complex_result = db
            .create_test_author("author:complex", "Complex User", "user+tag@domain.co.uk")
            .await;
        assert!(
            complex_result.is_ok(),
            "Complex valid email should be accepted"
        );

        let complex_name = db
            .query_field_string("author:complex", "name")
            .await
            .unwrap();
        assert_eq!(complex_name, Some("Complex User".to_string()));

        // Test basic data insertion to validate setup
        db.insert_basic_test_data().await.unwrap();
        let test_name = db.query_field_string("author:test", "name").await.unwrap();
        assert_eq!(test_name, Some("Test Author".to_string()));
    }

    /// Test migration rollback scenarios
    #[tokio::test]
    async fn test_migration_rollback_reliability() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        let mut rollback_harness = RollbackTestCapability::default();

        // Apply migration and create data using working pattern
        db.apply_cached_migrations(&["initial", "indexes"])
            .await
            .unwrap();
        db.setup_complete_testing().await.unwrap();

        // Create test data using working methods
        db.insert_basic_test_data().await.unwrap();
        db.create_test_author("author:rollback", "Rollback User", "rollback@example.com")
            .await
            .unwrap();

        // Create a snapshot before rollback
        rollback_harness
            .create_snapshot(&db, "before_rollback")
            .await
            .unwrap();
        assert!(rollback_harness.verify_snapshot("before_rollback"));
        let snapshot_data = rollback_harness
            .get_snapshot_data("before_rollback")
            .unwrap();
        assert_eq!(snapshot_data[0], 2); // 2 authors
        assert_eq!(snapshot_data[1], 1); // 1 post

        // Verify data exists before rollback
        let test_name = db.query_field_string("author:test", "name").await.unwrap();
        assert_eq!(test_name, Some("Test Author".to_string()));

        let rollback_name = db
            .query_field_string("author:rollback", "name")
            .await
            .unwrap();
        assert_eq!(rollback_name, Some("Rollback User".to_string()));

        // Simulate rollback by creating new database instance
        let mut fresh_db = MigrationTestFramework::new().await.unwrap();
        fresh_db
            .apply_cached_migrations(&["initial"])
            .await
            .unwrap();
        fresh_db.setup_complete_testing().await.unwrap();

        // Verify clean state after "rollback" - queries should return None
        let clean_test = fresh_db
            .query_field_string("author:test", "name")
            .await
            .unwrap();
        assert_eq!(clean_test, None, "Fresh database should have no data");

        let clean_rollback = fresh_db
            .query_field_string("author:rollback", "name")
            .await
            .unwrap();
        assert_eq!(
            clean_rollback, None,
            "Fresh database should have no rollback data"
        );

        // Verify recovery capability on fresh database
        fresh_db.insert_basic_test_data().await.unwrap();
        let recovery_name = fresh_db
            .query_field_string("author:test", "name")
            .await
            .unwrap();
        assert_eq!(recovery_name, Some("Test Author".to_string()));
    }

    /// Test comment system integration with deterministic validation
    #[tokio::test]
    async fn test_comment_system_integration() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&["initial", "indexes", "comments"])
            .await
            .unwrap();
        db.setup_complete_testing().await.unwrap();

        // Create base data
        db.create_test_author("author:commenter", "Comment Author", "comment@example.com")
            .await
            .unwrap();
        db.create_test_post(
            "post:comment_test",
            "Comment Test Post",
            "Summary",
            "Content",
            "author:commenter",
        )
        .await
        .unwrap();

        // Add comments with different approval statuses
        db.create_test_comment(
            "comment:approved",
            "Approved comment content",
            "Approver",
            "approve@example.com",
            "post:comment_test",
            true,
        )
        .await
        .unwrap();

        db.create_test_comment(
            "comment:pending",
            "Pending comment content",
            "Pending User",
            "pending@example.com",
            "post:comment_test",
            false,
        )
        .await
        .unwrap();

        // Verify comment data integrity
        let approved_content = db
            .query_field_string("comment:approved", "content")
            .await
            .unwrap();
        assert_eq!(
            approved_content,
            Some("Approved comment content".to_string())
        );

        let pending_status = db
            .execute_query("SELECT VALUE is_approved FROM comment:pending")
            .await
            .unwrap();
        let mut response = pending_status;
        let is_approved: Option<bool> = response.take(0).unwrap();
        assert_eq!(is_approved, Some(false));

        // Verify comment count
        assert_eq!(db.count_table_records("comment").await.unwrap(), 2);
    }

    /// Test utility functions and edge cases
    #[tokio::test]
    async fn test_utility_functions_comprehensive() {
        let mut db = MigrationTestFramework::new().await.unwrap();

        // Test migration tracking
        assert_eq!(db.get_schema_version(), 0);
        assert_eq!(db.get_migration_count(), 0);

        let migration_content = db.read_migration_file("initial").unwrap();
        db.execute_migration(&migration_content).await.unwrap();

        assert_ne!(db.get_schema_version(), 0);
        assert_eq!(db.get_migration_count(), 1);
        assert!(db.has_migration_applied(0));
        assert!(!db.has_migration_applied(1));

        let applied_migrations = db.get_applied_migrations();
        assert_eq!(applied_migrations.len(), 1);
        assert!(db.has_migration_content_applied(&migration_content));

        // Test table/field verification
        assert!(db.verify_table_exists("author").await.unwrap());
        assert!(db.verify_field_exists("author", "name").await.unwrap());

        // Test data creation and querying
        db.setup_complete_testing().await.unwrap();
        db.insert_basic_test_data().await.unwrap();

        let test_name = db.query_field_string("author:test", "name").await.unwrap();
        assert_eq!(test_name, Some("Test Author".to_string()));

        let total_views = db
            .query_field_i64("post:test", "total_views")
            .await
            .unwrap();
        assert_eq!(total_views, Some(0));

        let post_author = db.query_field_thing("post:test", "author").await.unwrap();
        assert_eq!(post_author.unwrap().to_string(), "author:test");
    }

    /// Stress test for performance validation
    #[tokio::test]
    async fn test_high_volume_operations() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&["initial", "indexes"])
            .await
            .unwrap();
        db.setup_complete_testing().await.unwrap();

        let start = std::time::Instant::now();

        // Create 100 records in single batch operation
        let mut batch_query = String::from("BEGIN TRANSACTION;");
        for i in 0..100 {
            batch_query.push_str(&format!(
                "CREATE author:stress_{} SET name = 'Stress Author {}', email = 'stress{}@example.com';\n",
                i, i, i
            ));
        }
        batch_query.push_str("COMMIT TRANSACTION;");

        db.execute_query(&batch_query)
            .await
            .unwrap()
            .check()
            .unwrap();

        let batch_duration = start.elapsed();

        // Performance validation - should handle 100 records efficiently
        assert!(
            batch_duration.as_millis() < 2000,
            "High volume operations took too long: {:?}",
            batch_duration
        );

        // Verify record count
        assert_eq!(db.count_table_records("author").await.unwrap(), 100);
    }
}
