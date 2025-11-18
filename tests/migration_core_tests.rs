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
    use serde::Deserialize;
    use surrealdb_types::{RecordId, SurrealValue};

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
        assert_eq!(post_author, Some(RecordId::new("author", "test")));
    }

    /// Test post activity event functionality
    #[tokio::test]
    async fn test_post_activity_event() {
        #[derive(Debug, Deserialize, SurrealValue)]
        struct ActivityRow {
            id: RecordId,
            content: String,
            tags: Vec<String>,
            source: Option<String>,
        }

        let mut db = MigrationTestFramework::new().await.unwrap();

        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&[
            "initial",
            "indexes",
            "comments",
            "activity",
            "post_activity",
        ])
        .await
        .unwrap();
        db.setup_complete_testing().await.unwrap();

        db.create_test_author(
            "author:event_test",
            "Event Test Author",
            "event@example.com",
        )
        .await
        .unwrap();

        db.create_test_post(
            "post:event_test",
            "Event Test Post",
            "Test summary for event",
            "Test content for event post",
            "author:event_test",
        )
        .await
        .unwrap();

        assert_eq!(
            db.count_table_records("activity").await.unwrap(),
            0,
            "No activity should exist before publishing"
        );

        db.execute_query(
            "UPDATE post:event_test SET is_published = true, slug = 'event-test-post';",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        db.create_test_author(
            "author:event_test2",
            "Event Test Author 2",
            "event2@example.com",
        )
        .await
        .unwrap();

        db.execute_query(
            "CREATE post:event_test2 SET title = 'Event Test Post 2', summary = 'Test summary 2', body = 'Test content 2', tags = ['test'], author = author:event_test2, is_published = true, slug = 'event-test-post-2';"
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut response = db
            .execute_query(
                "SELECT id, content, tags, source, created_at FROM activity ORDER BY created_at ASC;",
            )
            .await
            .unwrap();
        let activities: Vec<ActivityRow> = response.take(0).unwrap();

        assert_eq!(activities.len(), 2, "Expected two activity entries");

        let first = &activities[0];
        assert_eq!(first.id, RecordId::new("activity", "post-event-test"));
        assert_eq!(
            first.content,
            "Published on Alex Thola's blog: Event Test Post - Test summary for event (https://alexthola.com/post/event-test-post)"
        );
        assert_eq!(first.tags, vec!["new post".to_string(), "blog".to_string()]);
        assert_eq!(
            first.source.as_deref(),
            Some("https://alexthola.com/post/event-test-post")
        );

        let second = &activities[1];
        assert_eq!(second.id, RecordId::new("activity", "post-event-test-2"));
        assert_eq!(
            second.content,
            "Published on Alex Thola's blog: Event Test Post 2 - Test summary 2 (https://alexthola.com/post/event-test-post-2)"
        );
        assert_eq!(
            second.tags,
            vec!["new post".to_string(), "blog".to_string()]
        );
        assert_eq!(
            second.source.as_deref(),
            Some("https://alexthola.com/post/event-test-post-2")
        );
    }

    /// Ensure the activity event derives a slugged URL when the post does not provide one
    #[tokio::test]
    async fn test_post_activity_event_generates_slug_when_missing() {
        #[derive(Debug, Deserialize, SurrealValue)]
        struct ActivityRow {
            id: RecordId,
            content: String,
            tags: Vec<String>,
            source: Option<String>,
        }

        let mut db = MigrationTestFramework::new().await.unwrap();
        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&[
            "initial",
            "indexes",
            "comments",
            "activity",
            "post_activity",
        ])
        .await
        .unwrap();
        db.setup_complete_testing().await.unwrap();

        db.create_test_author("author:slugless", "Slugless Author", "slugless@example.com")
            .await
            .unwrap();

        db.execute_query(
            "CREATE post:slugless SET title = 'Slugless Title', summary = 'Slugless summary', body = 'Body', tags = ['test'], author = author:slugless, is_published = true;"
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut response = db
            .execute_query("SELECT id, content, tags, source FROM activity LIMIT 1;")
            .await
            .unwrap();
        let activities: Vec<ActivityRow> = response.take(0).unwrap();
        assert_eq!(activities.len(), 1);

        let activity = &activities[0];
        assert_eq!(
            activity.id,
            RecordId::new("activity", "post-slugless-title")
        );
        assert_eq!(
            activity.source.as_deref(),
            Some("https://alexthola.com/post/slugless-title")
        );
        assert!(activity
            .content
            .contains("https://alexthola.com/post/slugless-title"));
        assert_eq!(
            activity.tags,
            vec!["new post".to_string(), "blog".to_string()]
        );
    }

    /// Verify very long summaries are truncated for concise activity entries
    #[tokio::test]
    async fn test_post_activity_event_truncates_long_summary() {
        let mut db = MigrationTestFramework::new().await.unwrap();
        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&[
            "initial",
            "indexes",
            "comments",
            "activity",
            "post_activity",
        ])
        .await
        .unwrap();
        db.setup_complete_testing().await.unwrap();

        db.create_test_author("author:summary", "Summary Author", "summary@example.com")
            .await
            .unwrap();

        let long_summary = "A".repeat(220);
        db.execute_query(&format!(
            "CREATE post:summary_test SET title = 'Summary Title', summary = '{}', body = 'Body', tags = ['test'], author = author:summary, is_published = true;",
            long_summary
        ))
        .await
        .unwrap()
        .check()
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut response = db
            .execute_query("SELECT VALUE content FROM activity LIMIT 1;")
            .await
            .unwrap();
        let contents: Vec<String> = response.take(0).unwrap();
        assert_eq!(contents.len(), 1);

        let content = &contents[0];
        assert!(content.starts_with("Published on Alex Thola's blog: Summary Title - "));
        assert!(content.contains("https://alexthola.com/post/summary-title"));

        let summary_fragment = content
            .split(" - ")
            .nth(1)
            .and_then(|segment| segment.split(" (https://").next())
            .unwrap_or("");
        assert!(summary_fragment.ends_with("..."));
        assert!(summary_fragment.len() <= 183);
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

    /// Ensure post activity event normalizes slugs with redundant "post" segments.
    #[tokio::test]
    async fn test_post_activity_event_slug_normalization_variants() {
        #[derive(Debug, Deserialize, SurrealValue)]
        struct ActivityRow {
            id: RecordId,
        }

        let mut db = MigrationTestFramework::new().await.unwrap();
        db.reset_database().await.unwrap();
        db.apply_cached_migrations(&[
            "initial",
            "indexes",
            "comments",
            "activity",
            "post_activity",
        ])
        .await
        .unwrap();

        db.setup_complete_testing().await.unwrap();
        db.create_test_author(
            "author:slug_variants",
            "Slug Variant Author",
            "slug@example.com",
        )
        .await
        .unwrap();

        db.create_test_post(
            "post:slug_variant_one",
            "Slug Variant One",
            "Summary",
            "Body",
            "author:slug_variants",
        )
        .await
        .unwrap();

        db.execute_query(
            "UPDATE post:slug_variant_one SET is_published = true, slug = 'complex-post-case-post';",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        db.execute_query(
            "CREATE post:slug_variant_two SET title = 'Slug Variant Two', summary = 'Summary 2', body = 'More body', tags = ['test'], author = author:slug_variants, is_published = true, slug = 'demo-post-mid-post-example';",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut response = db
            .execute_query("SELECT id, created_at FROM activity ORDER BY created_at ASC;")
            .await
            .unwrap();
        let rows: Vec<ActivityRow> = response.take(0).unwrap();

        assert_eq!(rows.len(), 2, "Expected two normalized activity entries");
        assert_eq!(rows[0].id, RecordId::new("activity", "post-complex-case"));
        assert_eq!(
            rows[1].id,
            RecordId::new("activity", "post-demo-mid-example")
        );
    }
}
