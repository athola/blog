# Database Test Suite

This document explains the structure and usage of the database test suite. The suite is designed to validate database migrations and schema changes.

## Test Harness

The core of the suite is the test harness in `harness/mod.rs`. This harness abstracts away the repetitive tasks of setting up and tearing down database tests. Its features include:

-   **Migration Application:** Applies migrations from the `migrations/` directory in a specified order.
-   **Data Seeding:** Includes a `TestDataBuilder` for creating consistent test data (e.g., authors, posts).
-   **Assertions:** Provides helper functions, such as `count_table_records`, for verifying test outcomes.

## Test Organization

-   `migration_core_tests.rs`: Contains tests for the fundamental migration logic, ensuring that migrations apply correctly, constraints are enforced, and performance is acceptable.
-   `schema_evolution_tests.rs`: Contains tests for more complex scenarios, such as verifying that data is preserved correctly after a schema change.

## Usage Example

Here is a basic example of how to write a test using the harness.

```rust
#[tokio::test]
async fn test_author_creation() {
    // 1. Create a new instance of the test framework.
    let mut db = MigrationTestFramework::new().await.unwrap();

    // 2. Apply the necessary migrations.
    db.apply_cached_migrations(&["initial"]).await.unwrap();
    db.setup_complete_testing().await.unwrap();

    // 3. Use the TestDataBuilder to create test data.
    let authors = TestDataBuilder::authors();
    db.create_test_authors(&authors).await.unwrap();

    // 4. Assert that the data was created correctly.
    assert_eq!(db.count_table_records("author").await.unwrap(), 3);
}
```

## Performance Testing Example

The suite can also be used for simple performance tests, like measuring the execution time of migrations.

```rust
#[tokio::test]
async fn test_migration_performance() {
    let start = std::time::Instant::now();

    // Apply the migrations to be measured.
    db.apply_cached_migrations(&["initial", "indexes"]).await.unwrap();

    let duration = start.elapsed();

    // Assert that the duration is within an acceptable range.
    assert!(duration.as_millis() < 1000, "Migrations should apply in less than 1 second.");
}
```
