# SurrealDB Migration Test Suite

This test suite validates SurrealDB schema migrations and database operations.

## Architecture

### `surrealdb_test_harness.rs`

A testing framework for SurrealDB operations. It provides:

-   **Migration Caching**: Pre-loads migration files for faster execution.
-   **Batch Operations**: Executes multiple operations in a single query.
-   **Schema Management**: Automates permission and field definition setup.
-   **Data Verification**: Provides helpers for table counting and field validation.

### `migration_core_tests.rs`

Tests core migration functionality and performance. It covers:

-   Migration chain performance.
-   Batch operations.
-   Schema evolution.
-   Constraint validation.

### `schema_evolution_tests.rs`

Tests complex integration scenarios and schema evolution, including:

-   Multi-stage migrations.
-   Data preservation across migrations.
-   Relationship validation.

## Usage

### Basic Test Setup

```rust
#[tokio::test]
asyn fn test_example() {
    let mut db = MigrationTestFramework::new().await.unwrap();
    db.apply_cached_migrations(&["initial"]).await.unwrap();
    db.setup_complete_testing().await.unwrap();

    let authors = TestDataBuilder::authors();
    db.create_test_authors(&authors).await.unwrap();

    assert_eq!(db.count_table_records("author").await.unwrap(), 3);
}
```

### Performance Testing

```rust
#[tokio::test]
asyn fn test_performance() {
    let start = std::time::Instant::now();

    db.apply_cached_migrations(&["initial", "indexes"]).await.unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Should complete in < 1000ms");
}
```
