use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{IndexedResults, Result as SurrealResult, Surreal};
use surrealdb_types::{RecordId, SurrealValue};

#[derive(Debug, Deserialize, SurrealValue)]
struct CountResult {
    count: i64,
}

/// SurrealDB test harness for migration and schema management testing
///
/// Provides a comprehensive testing framework for SurrealDB operations including:
/// - Migration execution with caching and batch operations
/// - Schema definition and constraint validation
/// - Data creation and verification utilities
/// - Performance testing and rollback capabilities
pub struct MigrationTestFramework {
    pub db: Surreal<Db>,
    applied_migrations: Vec<String>,
    schema_version: u64,
    migration_cache: HashMap<&'static str, String>,
}

impl MigrationTestFramework {
    pub async fn new() -> SurrealResult<Self> {
        let db = Surreal::new::<Mem>(()).await?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let namespace = format!("test_ns_{}", timestamp);
        let database = format!("test_db_{}", timestamp);

        db.use_ns(&namespace).use_db(&database).await?;

        // Pre-cache migrations for optimal performance
        let migrations = [
            ("initial", "migrations/0001_initial_schema.surql"),
            ("indexes", "migrations/0002_add_indexes.surql"),
            ("comments", "migrations/0003_add_comments.surql"),
            ("activity", "migrations/0004_add_activity_table.surql"),
            (
                "post_activity",
                "migrations/0005_add_post_activity_event.surql",
            ),
        ];

        let mut migration_cache = HashMap::new();
        for (key, file) in migrations {
            if let Ok(content) = std::fs::read_to_string(file) {
                migration_cache.insert(key, content);
            }
        }

        Ok(MigrationTestFramework {
            db,
            applied_migrations: Vec::new(),
            schema_version: 0,
            migration_cache,
        })
    }

    #[allow(dead_code)]
    pub fn read_migration_file(&self, key: &str) -> std::io::Result<String> {
        let file_path = match key {
            "initial" => "migrations/0001_initial_schema.surql",
            "indexes" => "migrations/0002_add_indexes.surql",
            "comments" => "migrations/0003_add_comments.surql",
            "activity" => "migrations/0004_add_activity_table.surql",
            "post_activity" => "migrations/0005_add_post_activity_event.surql",
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Migration key not found",
                ));
            }
        };
        std::fs::read_to_string(file_path)
    }

    /// Execute migration with performance optimization and version tracking
    pub async fn execute_migration(&mut self, migration_content: &str) -> SurrealResult<()> {
        self.db.query(migration_content).await?.check()?;

        self.applied_migrations.push(migration_content.to_string());

        let mut hasher = DefaultHasher::new();
        self.applied_migrations.hash(&mut hasher);
        self.schema_version = hasher.finish();

        Ok(())
    }

    /// Apply cached migrations efficiently with single database round trip per migration
    pub async fn apply_cached_migrations(&mut self, keys: &[&str]) -> SurrealResult<()> {
        for key in keys {
            if let Some(migration) = self.migration_cache.get(key).cloned() {
                self.execute_migration(&migration).await?;
            } else {
                return Err(surrealdb::Error::Query(format!(
                    "Migration not found: {}",
                    key
                )));
            }
        }
        Ok(())
    }

    /// Apply migrations from files with performance optimization
    #[allow(dead_code)]
    pub async fn apply_migrations(&mut self, migration_files: &[&str]) -> SurrealResult<()> {
        for migration_file in migration_files {
            let migration_content = std::fs::read_to_string(migration_file)
                .unwrap_or_else(|_| panic!("Failed to read {}", migration_file));
            self.execute_migration(&migration_content).await?;
        }
        Ok(())
    }

    /// Enable full permissions with single batch operation
    pub async fn enable_test_permissions(&self) -> SurrealResult<()> {
        self.db
            .query(
                r#"
            DEFINE TABLE OVERWRITE author TYPE NORMAL SCHEMAFULL
                PERMISSIONS FOR select, create, update, delete FULL;
            DEFINE TABLE OVERWRITE post TYPE NORMAL SCHEMAFULL
                PERMISSIONS FOR select, create, update, delete FULL;
            DEFINE TABLE OVERWRITE comment TYPE NORMAL SCHEMAFULL
                PERMISSIONS FOR select, create, update, delete FULL;
            DEFINE TABLE OVERWRITE activity TYPE NORMAL SCHEMAFULL
                PERMISSIONS FOR select, create, update, delete FULL;
        "#,
            )
            .await?
            .check()?;

        Ok(())
    }

    /// Define all essential fields in optimized batches to minimize database round trips
    pub async fn define_all_fields(&self) -> SurrealResult<()> {
        self.db
            .query(
                r#"
            DEFINE FIELD OVERWRITE name ON author TYPE string ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE email ON author TYPE string ASSERT string::is_email($value);
            DEFINE FIELD OVERWRITE bio ON author TYPE option<string>;
            
            DEFINE FIELD OVERWRITE title ON post TYPE string ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE summary ON post TYPE string ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE body ON post TYPE string ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE slug ON post TYPE option<string>;
            DEFINE FIELD OVERWRITE tags ON post TYPE array<string> ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE author ON post TYPE record<author> ASSERT $value != NONE;
            DEFINE FIELD OVERWRITE read_time ON post TYPE option<int>;
            DEFINE FIELD OVERWRITE total_views ON post TYPE int DEFAULT 0;
            DEFINE FIELD OVERWRITE created_at ON post TYPE datetime DEFAULT time::now();
            DEFINE FIELD OVERWRITE updated_at ON post TYPE datetime VALUE time::now();
            DEFINE FIELD OVERWRITE is_published ON post TYPE bool DEFAULT false;
            DEFINE FIELD OVERWRITE header_image ON post TYPE option<string>;
            DEFINE FIELD OVERWRITE show_cta ON post TYPE bool DEFAULT false;
        "#,
            )
            .await?
            .check()?;

        Ok(())
    }

    /// Optimized complete testing setup in single operation
    pub async fn setup_complete_testing(&self) -> SurrealResult<()> {
        self.enable_test_permissions().await?;
        self.define_all_fields().await?;
        Ok(())
    }

    /// Clean database state for test isolation
    pub async fn reset_database(&self) -> SurrealResult<()> {
        let tables = [
            "comment",
            "post",
            "author",
            "reference",
            "script_migration",
            "activity",
        ];
        for table in tables {
            let _ = self.db.query(format!("REMOVE TABLE {};", table)).await;
        }
        Ok(())
    }

    /// Create test authors using optimized batch operations
    pub async fn create_test_authors(&self, authors: &[(&str, &str, &str)]) -> SurrealResult<()> {
        let mut batch_query = String::new();
        for (id, name, email) in authors {
            batch_query.push_str(&format!(
                "CREATE {} SET name = '{}', email = '{}';\n",
                id, name, email
            ));
        }
        if !batch_query.is_empty() {
            self.db.query(batch_query).await?.check()?;
        }
        Ok(())
    }

    /// Create test posts using optimized batch operations
    #[allow(dead_code)]
    pub async fn create_test_posts(
        &self,
        posts: &[(&str, &str, &str, &str, &str)],
    ) -> SurrealResult<()> {
        let mut batch_query = String::new();
        for (id, title, summary, body, author_id) in posts {
            batch_query.push_str(&format!(
                "CREATE {} SET title = '{}', summary
                    = '{}', body = '{}', tags = ['test'], author = {}
                    ;\n",
                id, title, summary, body, author_id
            ));
        }
        if !batch_query.is_empty() {
            self.db.query(batch_query).await?.check()?;
        }
        Ok(())
    }

    /// Create single test author
    pub async fn create_test_author(&self, id: &str, name: &str, email: &str) -> SurrealResult<()> {
        self.db
            .query(format!(
                "CREATE {} SET name = '{}', email = '{}'",
                id, name, email
            ))
            .await?
            .check()?;
        Ok(())
    }

    /// Create single test post
    pub async fn create_test_post(
        &self,
        id: &str,
        title: &str,
        summary: &str,
        body: &str,
        author_id: &str,
    ) -> SurrealResult<()> {
        self.db
            .query(format!(
            "CREATE {} SET title = '{}', summary = '{}', body = '{}', tags = ['test'], author = {}",
            id, title, summary, body, author_id
        ))
            .await?
            .check()?;
        Ok(())
    }

    /// Create custom post with specific tags
    #[allow(dead_code)]
    pub async fn create_custom_post(
        &self,
        id: &str,
        title: &str,
        summary: &str,
        body: &str,
        tags: &[&str],
        author_id: &str,
    ) -> SurrealResult<()> {
        let tags_str = tags
            .iter()
            .map(|t| format!("'{}'", t))
            .collect::<Vec<_>>()
            .join(", ");
        self.db
            .query(format!(
                "CREATE {} SET title = '{}', summary = '{}', body = '{}', tags = [{}], author = {}",
                id, title, summary, body, tags_str, author_id
            ))
            .await?
            .check()?;
        Ok(())
    }

    /// Create test comment
    pub async fn create_test_comment(
        &self,
        id: &str,
        content: &str,
        author_name: &str,
        author_email: &str,
        post_id: &str,
        is_approved: bool,
    ) -> SurrealResult<()> {
        self.db.query(format!(
            "CREATE {} SET content = '{}', author_name = '{}', author_email = '{}', post_id = {}, is_approved = {}",
            id, content, author_name, author_email, post_id, is_approved
        )).await?.check()?;
        Ok(())
    }

    /// Optimized basic test data setup with single batch operation
    pub async fn insert_basic_test_data(&self) -> SurrealResult<()> {
        self.setup_complete_testing().await?;
        let test_body = TestDataBuilder::content_with_word_count(15);

        self.db.query(format!(
            "CREATE author:test SET name = 'Test Author', email = 'test@example.com';
             CREATE post:test SET title = 'Test Post', summary = 'Test summary', body = '{}', tags = ['test'], author = author:test;",
            test_body
        )).await?.check()?;

        Ok(())
    }

    /// Optimized data verification with single query
    pub async fn count_table_records(&self, table: &str) -> SurrealResult<i64> {
        let mut result = self
            .db
            .query(format!("SELECT count() FROM {} GROUP ALL", table))
            .await?;
        let count: Option<CountResult> = result.take(0)?;
        Ok(count.map(|c| c.count).unwrap_or(0))
    }

    /// Verify table exists using efficient query
    pub async fn verify_table_exists(&self, table_name: &str) -> SurrealResult<bool> {
        let result = self
            .db
            .query(format!("SELECT * FROM {} LIMIT 0", table_name))
            .await;
        Ok(result.is_ok())
    }

    /// Verify field exists using efficient query
    pub async fn verify_field_exists(&self, table: &str, field: &str) -> SurrealResult<bool> {
        let result = self
            .db
            .query(format!("SELECT {} FROM {} LIMIT 0", field, table))
            .await;
        Ok(result.is_ok())
    }

    /// Query string field value
    pub async fn query_field_string(
        &self,
        table_record: &str,
        field: &str,
    ) -> SurrealResult<Option<String>> {
        let mut result = self
            .db
            .query(format!("SELECT VALUE {} FROM {}", field, table_record))
            .await?;
        result.take(0)
    }

    /// Query integer field value
    pub async fn query_field_i64(
        &self,
        table_record: &str,
        field: &str,
    ) -> SurrealResult<Option<i64>> {
        let mut result = self
            .db
            .query(format!("SELECT VALUE {} FROM {}", field, table_record))
            .await?;
        result.take(0)
    }

    /// Query Thing field value (for record references)
    pub async fn query_field_thing(
        &self,
        table_record: &str,
        field: &str,
    ) -> SurrealResult<Option<RecordId>> {
        let mut result = self
            .db
            .query(format!("SELECT VALUE {} FROM {}", field, table_record))
            .await?;
        result.take(0)
    }

    /// Execute raw query for complex operations
    pub async fn execute_query(&self, query: &str) -> SurrealResult<IndexedResults> {
        self.db.query(query).await
    }

    // Migration tracking functions for schema versioning
    pub fn get_schema_version(&self) -> u64 {
        self.schema_version
    }

    pub fn get_applied_migrations(&self) -> &[String] {
        &self.applied_migrations
    }

    pub fn get_migration_count(&self) -> usize {
        self.applied_migrations.len()
    }

    pub fn has_migration_applied(&self, migration_index: usize) -> bool {
        self.applied_migrations.len() > migration_index
    }

    pub fn has_migration_content_applied(&self, migration_content: &str) -> bool {
        self.applied_migrations
            .contains(&migration_content.to_string())
    }
}

/// Rollback testing utilities for migration failure scenarios
pub struct RollbackTestCapability {
    #[allow(dead_code)]
    snapshots: HashMap<String, Vec<i64>>,
}

impl Default for RollbackTestCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl RollbackTestCapability {
    pub fn new() -> Self {
        RollbackTestCapability {
            snapshots: HashMap::new(),
        }
    }

    /// Create data snapshot for rollback testing  
    #[allow(dead_code)]
    pub async fn create_snapshot(
        &mut self,
        db: &MigrationTestFramework,
        snapshot_name: &str,
    ) -> SurrealResult<()> {
        let tables = ["author", "post", "reference", "comment"];
        let mut snapshot_data = Vec::new();

        for table in tables {
            if let Ok(count) = db.count_table_records(table).await {
                snapshot_data.push(count);
            }
        }

        self.snapshots
            .insert(snapshot_name.to_string(), snapshot_data);
        Ok(())
    }

    /// Verify snapshot integrity
    #[allow(dead_code)]
    pub fn verify_snapshot(&self, snapshot_name: &str) -> bool {
        self.snapshots.contains_key(snapshot_name)
    }

    /// Get snapshot record counts for validation
    #[allow(dead_code)]
    pub fn get_snapshot_data(&self, snapshot_name: &str) -> Option<&Vec<i64>> {
        self.snapshots.get(snapshot_name)
    }
}

/// Test data generators for consistent SurrealDB testing
pub struct TestDataBuilder;

impl TestDataBuilder {
    /// Standard test authors for consistent testing across all scenarios
    pub fn authors() -> [(&'static str, &'static str, &'static str); 3] {
        [
            ("author:john", "John Doe", "john@example.com"),
            ("author:jane", "Jane Smith", "jane@example.com"),
            ("author:bob", "Bob Wilson", "bob@example.com"),
        ]
    }

    /// Standard test posts for consistent testing across all scenarios
    #[allow(dead_code)]
    pub fn posts() -> [(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ); 3] {
        [
            (
                "post:tech",
                "Tech Trends",
                "Technology summary",
                "Detailed tech content",
                "author:john",
            ),
            (
                "post:science",
                "Science News",
                "Science summary",
                "Scientific discoveries",
                "author:jane",
            ),
            (
                "post:life",
                "Life Tips",
                "Life summary",
                "Lifestyle advice",
                "author:bob",
            ),
        ]
    }

    /// Generate test content with specific word count for performance testing
    pub fn content_with_word_count(word_count: usize) -> String {
        let base_words = ["This", "is", "test", "content", "for", "validation"];
        let mut result = String::with_capacity(word_count * 5); // Pre-allocate capacity

        for i in 0..word_count {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(base_words[i % base_words.len()]);
        }
        result.push('.');
        result
    }
}
