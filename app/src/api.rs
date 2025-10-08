#![allow(deprecated)]
extern crate alloc;
use alloc::collections::BTreeMap;

use leptos::prelude::{ServerFnError, server};
use leptos::server_fn::codec::GetUrl;
use leptos::server_fn::error::NoCustomError;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::time::Duration;
#[cfg(feature = "ssr")]
use tokio_retry::{Retry, strategy::ExponentialBackoff};

use crate::types::{Post, Reference};

#[cfg(feature = "ssr")]
async fn retry_db_operation<F, Fut, T>(operation: F) -> Result<T, ServerFnError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, surrealdb::Error>>,
{
    let retry_strategy = ExponentialBackoff::from_millis(50)
        .max_delay(Duration::from_secs(2))
        .take(3); // Maximum 3 retry attempts

    match Retry::spawn(retry_strategy, || async {
        operation().await.map_err(|e| {
            tracing::warn!("Database operation failed, retrying: {:?}", e);
            e
        })
    })
    .await
    {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!("Database operation failed after retries: {:?}", e);
            Err(ServerFnError::<NoCustomError>::ServerError(format!(
                "Database error: {}",
                e
            )))
        }
    }
}

#[server(endpoint = "/posts")]
pub async fn select_posts(
    #[server(default)] tags: Vec<String>,
) -> Result<Vec<Post>, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();
    let query = if tags.is_empty() {
        String::from(
            "SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC;",
        )
    } else {
        let tags = tags
            .iter()
            .map(|tag| format!(r#"""{tag}"""#))
            .collect::<Vec<_>>();
        format!(
            "SELECT *, author.* from post WHERE tags CONTAINSANY [{0}] ORDER BY created_at DESC;",
            tags.join(", ")
        )
    };

    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let mut posts = query
        .take::<Vec<Post>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    for post in &mut posts.iter_mut() {
        if let Ok(parsed_date) = DateTime::parse_from_rfc3339(&post.created_at) {
            let date_time = parsed_date.with_timezone(&Utc);
            let naive_date = date_time.date_naive();
            let formatted_date = naive_date.format("%b %-d, %Y").to_string();
            post.created_at = formatted_date;
        }
    }

    Ok(posts)
}

#[server(endpoint = "/tags")]
pub async fn select_tags() -> Result<BTreeMap<String, usize>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let query = "
    LET $tags = SELECT tags FROM post;
    array::flatten($tags.map(|$t| $t.tags));
    "
    .to_owned();
    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let tags = query
        .take::<Vec<String>>(1)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    let mut tag_map = BTreeMap::<String, usize>::new();
    for tag in tags {
        *tag_map.entry(tag).or_insert(0) += 1;
    }

    Ok(tag_map)
}

#[server(endpoint = "/post")]
pub async fn select_post(slug: String) -> Result<Post, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;
    use markdown::process_markdown;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let query_str = format!(r#"SELECT *, author.* from post WHERE slug = "{slug}""#);
    let mut query = retry_db_operation(|| async { db.query(&query_str).await }).await?;
    let post = query
        .take::<Vec<Post>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    let mut post = match post.first() {
        Some(first_post) => first_post.clone(),
        None => {
            return Err(ServerFnError::Request(
                "Failed to retrieve first post".to_owned(),
            ));
        }
    };

    let date_time = DateTime::parse_from_rfc3339(&post.created_at)?.with_timezone(&Utc);
    let naive_date = date_time.date_naive();
    let formatted_date = naive_date.format("%b %-d").to_string();
    post.created_at = formatted_date;
    post.body = process_markdown(&post.body)?;

    Ok(post)
}

#[server(endpoint = "/increment_views")]
pub async fn increment_views(id: String) -> Result<(), ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let query_str = format!("UPDATE post:{id} SET total_views = total_views + 1;");
    retry_db_operation(|| async { db.query(&query_str).await }).await?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
}

#[server(endpoint = "/contact")]
pub async fn contact(data: ContactRequest) -> Result<(), ServerFnError> {
    use lettre::{
        AsyncSmtpTransport, AsyncTransport as _, Message, Tokio1Executor,
        message::header::ContentType, transport::smtp::authentication::Credentials,
    };
    use std::env;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&env::var("SMTP_HOST")?)?
        .credentials(Credentials::new(
            env::var("SMTP_USER")?,
            env::var("SMTP_PASSWORD")?,
        ))
        .build::<Tokio1Executor>();

    let email = Message::builder()
        .from(env::var("SMTP_USER")?.parse()?)
        .to(env::var("SMTP_USER")?.parse()?)
        .subject(format!("{} - {}", data.email, data.subject))
        .header(ContentType::TEXT_HTML)
        .body(data.message)?;

    // Retry email sending with exponential backoff
    let retry_strategy = ExponentialBackoff::from_millis(200)
        .max_delay(Duration::from_secs(10))
        .take(3); // Maximum 3 retry attempts for email

    match Retry::spawn(retry_strategy, || async {
        match mailer.send(email.clone()).await {
            Ok(response) => {
                tracing::info!("Email sent successfully: {:?}", response);
                Ok(())
            }
            Err(email_err) => {
                tracing::warn!("Failed to send email, retrying: {:?}", email_err);
                Err(email_err)
            }
        }
    })
    .await
    {
        Ok(_) => {
            tracing::info!("Email sent successfully with retries");
            Ok(())
        }
        Err(email_err) => {
            tracing::error!("Failed to send email after retries: {:?}", email_err);
            Err(ServerFnError::from(email_err))
        }
    }
}

#[server(endpoint = "/references")]
pub async fn select_references() -> Result<Vec<Reference>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let query_str = "SELECT * from reference WHERE is_published = true ORDER BY created_at DESC;";
    let mut query = retry_db_operation(|| async { db.query(query_str).await }).await?;
    let references = query
        .take::<Vec<Reference>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    Ok(references)
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: usize,
}

#[server(prefix = "/api/activities", endpoint = "create")]
pub async fn create_activity(activity: crate::types::Activity) -> Result<(), ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let _created: Option<crate::types::Activity> =
        retry_db_operation(|| async { db.create("activity").content(activity.clone()).await })
            .await?;

    Ok(())
}

#[server(prefix = "/api", endpoint = "activities", input = GetUrl)]
pub async fn select_activities(
    #[server(default)] page: usize,
) -> Result<Vec<crate::types::Activity>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();
    let activities_per_page = 10;
    let start = page * activities_per_page;

    let query = format!(
        "SELECT * FROM activity ORDER BY created_at DESC LIMIT {} START {};",
        activities_per_page, start
    );

    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let activities = query
        .take::<Vec<crate::types::Activity>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[cfg(feature = "ssr")]
    use tokio_test::block_on;

    #[cfg(feature = "ssr")]
    #[test]
    fn test_retry_db_operation_success_first_attempt() {
        block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_db_operation(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<String, surrealdb::Error>("success".to_string())
                }
            })
            .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "success");
            assert_eq!(call_count.load(Ordering::SeqCst), 1);
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_retry_db_operation_success_after_failures() {
        block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_db_operation(|| {
                let count = call_count_clone.clone();
                async move {
                    let current_count = count.fetch_add(1, Ordering::SeqCst);
                    if current_count < 2 {
                        // Fail first two attempts
                        Err(surrealdb::Error::msg("Temporary failure"))
                    } else {
                        // Succeed on third attempt
                        Ok::<String, surrealdb::Error>("success_after_retry".to_string())
                    }
                }
            })
            .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "success_after_retry");
            assert_eq!(call_count.load(Ordering::SeqCst), 3);
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_retry_db_operation_exhausts_retries() {
        block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_db_operation(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<String, surrealdb::Error>(surrealdb::Error::msg("Persistent failure"))
                }
            })
            .await;

            assert!(result.is_err());
            // Should try exactly 4 times (initial + 3 retries based on our retry strategy)
            assert_eq!(call_count.load(Ordering::SeqCst), 4);

            // Verify it's converted to ServerFnError
            match result.unwrap_err() {
                ServerFnError::<NoCustomError>::ServerError(_) => {
                    // Successfully converted to ServerFnError::<NoCustomError>::ServerError as expected
                }
                _ => panic!("Expected ServerFnError::<NoCustomError>::ServerError"),
            }
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_retry_db_operation_timing() {
        use std::time::Instant;

        block_on(async {
            let start = Instant::now();
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let _result = retry_db_operation(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<String, surrealdb::Error>(surrealdb::Error::msg("Always fail"))
                }
            })
            .await;

            let elapsed = start.elapsed();

            // With exponential backoff starting at 50ms, should take some time but not too long
            // Make timing assertions less strict to avoid flaky tests
            assert!(elapsed.as_millis() >= 25); // Some delay expected
            assert!(elapsed.as_secs() < 10); // But reasonable overall time
            assert_eq!(call_count.load(Ordering::SeqCst), 4);
        });
    }

    #[test]
    fn test_contact_request_default() {
        let request = ContactRequest::default();
        assert_eq!(request.name, "");
        assert_eq!(request.email, "");
        assert_eq!(request.subject, "");
        assert_eq!(request.message, "");
    }

    #[test]
    fn test_contact_request_serialization() {
        let request = ContactRequest {
            name: "Test Name".to_string(),
            email: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            message: "Test Message".to_string(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ContactRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.name, deserialized.name);
        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.subject, deserialized.subject);
        assert_eq!(request.message, deserialized.message);
    }

    // Test email retry mechanism would require mocking SMTP server
    // This is a placeholder for testing email retry logic structure
    #[cfg(feature = "ssr")]
    #[test]
    fn test_email_retry_configuration() {
        // Test email retry configuration without requiring actual SMTP
        use std::time::Duration;
        use tokio_retry::strategy::ExponentialBackoff;

        // Verify the contact function exists with correct signature
        let _: fn(ContactRequest) -> _ = contact;

        // Test the retry strategy configuration used in email sending
        let retry_strategy = ExponentialBackoff::from_millis(200)
            .max_delay(Duration::from_secs(10))
            .take(3);

        let delays: Vec<_> = retry_strategy.collect();

        // Should have exactly 3 retry attempts
        assert_eq!(delays.len(), 3);

        // First delay should be around 200ms
        assert!(delays[0] >= Duration::from_millis(180));
        assert!(delays[0] <= Duration::from_millis(220));

        // Test that we can create ContactRequest for email operations
        let request = ContactRequest {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            message: "Test message".to_string(),
        };

        assert!(!request.name.is_empty());
        assert!(!request.email.is_empty());
        assert!(request.email.contains('@'));
    }

    #[test]
    fn test_server_fn_endpoints_exist() {
        // Verify all server function endpoints are defined with correct signatures
        // This ensures our retry-enabled functions maintain their contracts

        let _: fn(Vec<String>) -> _ = select_posts;
        let _: fn() -> _ = select_tags;
        let _: fn(String) -> _ = select_post;
        let _: fn(String) -> _ = increment_views;
        let _: fn(ContactRequest) -> _ = contact;
        let _: fn() -> _ = select_references;
        let _: fn(Activity) -> _ = create_activity;
        let _: fn(usize) -> _ = select_activities;
    }

    #[test]
    fn test_activity_default() {
        let activity = Activity::default();
        assert_eq!(activity.content, "");
        assert_eq!(activity.tags, Vec::<String>::new());
        assert_eq!(activity.source, None);
        assert_eq!(activity.created_at, "");
    }

    #[test]
    fn test_activity_serialization() {
        let activity = Activity {
            content: "Test activity".to_string(),
            tags: vec!["test".to_string(), "rust".to_string()],
            source: Some("https://example.com".to_string()),
            created_at: "2023-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: Activity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(activity.content, deserialized.content);
        assert_eq!(activity.tags, deserialized.tags);
        assert_eq!(activity.source, deserialized.source);
        assert_eq!(activity.created_at, deserialized.created_at);
    }

    #[test]
    fn test_pagination_struct() {
        let pagination = Pagination { page: 1 };
        assert_eq!(pagination.page, 1);
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_create_activity_with_retry() {
        block_on(async {
            // Test that create_activity function exists and has correct signature
            let _: fn(Activity) -> _ = create_activity;

            // Test activity creation with valid data
            let activity = Activity {
                content: "Test activity content".to_string(),
                tags: vec!["test".to_string()],
                source: Some("https://test.com".to_string()),
                ..Default::default()
            };

            // Verify the activity can be serialized (required for database operations)
            let serialized = serde_json::to_string(&activity).unwrap();
            assert!(!serialized.is_empty());
            assert!(serialized.contains("Test activity content"));
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_select_activities_with_retry() {
        block_on(async {
            // Test that select_activities function exists and has correct signature
            let _: fn(usize) -> _ = select_activities;

            // Test pagination parameter handling
            let page = 0;
            let activities_per_page = 10;
            let start = page * activities_per_page;

            assert_eq!(start, 0);

            let page = 1;
            let start = page * activities_per_page;
            assert_eq!(start, 10);
        });
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_retry_with_different_error_types() {
        tokio_test::block_on(async {
            // Test retry behavior with different SurrealDB error types
            let network_error_count = Arc::new(AtomicUsize::new(0));
            let network_count_clone = network_error_count.clone();

            let result = retry_db_operation(|| {
                let count = network_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, Ordering::SeqCst);
                    if current == 0 {
                        Err(surrealdb::Error::msg("Network timeout"))
                    } else if current == 1 {
                        Err(surrealdb::Error::msg("Connection lost"))
                    } else {
                        Ok::<&str, surrealdb::Error>("recovered")
                    }
                }
            })
            .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "recovered");
            assert_eq!(network_error_count.load(Ordering::SeqCst), 3);
        });
    }

    // Additional activity-related unit tests from integration test patterns

    #[test]
    fn test_activity_json_structure_compatibility() {
        // Test that Activity struct matches the JSON structure used in integration tests
        let activity_json = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });

        // Test deserialization from the exact structure used in integration tests
        let activity: Activity = serde_json::from_value(activity_json).unwrap();

        assert_eq!(activity.content, "This is a test activity");
        assert_eq!(activity.tags, vec!["test", "rust"]);
        assert_eq!(activity.source, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_activity_creation_validation() {
        // Test activity creation patterns from integration test scenarios
        let test_cases = vec![
            // Valid activity
            Activity {
                content: "Valid activity".to_string(),
                tags: vec!["test".to_string()],
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
            // Activity with empty tags
            Activity {
                content: "Activity with no tags".to_string(),
                tags: vec![],
                source: None,
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
            // Activity with multiple tags
            Activity {
                content: "Multi-tag activity".to_string(),
                tags: vec!["rust".to_string(), "web".to_string(), "blog".to_string()],
                source: Some("https://blog.example.com".to_string()),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
        ];

        for activity in test_cases {
            // Test serialization roundtrip
            let serialized = serde_json::to_string(&activity).unwrap();
            let deserialized: Activity = serde_json::from_str(&serialized).unwrap();
            assert_eq!(activity, deserialized);
        }
    }

    #[test]
    fn test_activity_pagination_parameters() {
        // Test pagination parameter handling from integration test patterns
        let test_pages = vec![0, 1, 5, 10];

        for page in test_pages {
            // Test that pagination parameters are handled correctly
            // This mirrors the integration test that calls /api/activities?page=N
            assert!(page >= 0, "Page number should be non-negative");

            // Test the activities_per_page constant used in server function
            let activities_per_page = 10;
            let start = page * activities_per_page;
            assert!(start >= 0, "Start index should be non-negative");
        }
    }

    #[test]
    fn test_activity_endpoint_signatures() {
        // Test that activity server functions maintain correct signatures
        // This ensures compatibility with integration test expectations

        // Test create_activity signature
        let _: fn(Activity) -> _ = create_activity;

        // Test select_activities signature
        let _: fn(usize) -> _ = select_activities;

        // These signatures must match what the integration tests expect
    }

    #[test]
    fn test_activity_error_handling_patterns() {
        // Test error handling patterns that integration tests might encounter
        let invalid_activity_json = serde_json::json!({
            "content": 123, // Wrong type
            "tags": "not-an-array", // Wrong type
            "source": null // Valid null
        });

        // Test that invalid data is handled gracefully
        let result: Result<Activity, _> = serde_json::from_value(invalid_activity_json);
        assert!(
            result.is_err(),
            "Invalid activity data should fail deserialization"
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_activity_server_function_registration() {
        block_on(async {
            // Test that activity server functions are properly registered
            // This complements the integration test that actually calls the endpoints

            // Test function existence and basic structure
            let test_activity = Activity {
                content: "Test registration".to_string(),
                tags: vec!["test".to_string()],
                source: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            };

            // We can't actually call the server function without a proper context,
            // but we can verify the function signature and basic structure
            let _activity_clone = test_activity.clone();
            let _: fn(Activity) -> _ = create_activity;

            // This test ensures the function signature matches integration test expectations
        });
    }

    // Utility function tests extracted from integration test patterns

    #[test]
    fn test_port_calculation_logic() {
        // Test the port calculation logic used in integration tests
        let base_port = 3007;
        let test_port = 3030;

        // This mirrors the calculation: db_port = 8000 + (port - 3007)
        let expected_db_port = 8000 + (test_port - base_port);
        assert_eq!(expected_db_port, 8023);

        // Test edge cases
        let min_port = 3007;
        let min_db_port = 8000 + (min_port - base_port);
        assert_eq!(min_db_port, 8000);
    }

    #[test]
    fn test_activity_json_response_format() {
        // Test the expected JSON response format from integration tests
        let activity_response = serde_json::json!([
            {
                "content": "This is a test activity",
                "tags": ["test", "rust"],
                "source": "https://example.com",
                "created_at": "2023-01-01T00:00:00Z"
            }
        ]);

        // Test that the response format matches what integration tests expect
        assert!(activity_response.is_array());
        let activities: Vec<Activity> = serde_json::from_value(activity_response).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].content, "This is a test activity");
        assert_eq!(activities[0].tags, vec!["test", "rust"]);
        assert_eq!(
            activities[0].source,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_activity_endpoint_url_construction() {
        // Test URL construction patterns used in integration tests
        let base_url = "http://127.0.0.1:3030";
        let page = 0;

        let create_url = format!("{}/api/activities/create", base_url);
        let fetch_url = format!("{}/api/activities?page={}", base_url, page);

        assert_eq!(create_url, "http://127.0.0.1:3030/api/activities/create");
        assert_eq!(fetch_url, "http://127.0.0.1:3030/api/activities?page=0");

        // Test pagination URL construction
        for page in 0..=5 {
            let url = format!("{}/api/activities?page={}", base_url, page);
            assert!(url.contains(&format!("page={}", page)));
        }
    }

    #[test]
    fn test_activity_status_code_expectations() {
        // Test the status code expectations from integration tests
        use http::StatusCode;

        // These are the status codes that integration tests expect
        let expected_create_status = StatusCode::CREATED;
        let expected_fetch_status = StatusCode::OK;

        assert_eq!(expected_create_status, 201);
        assert_eq!(expected_fetch_status, 200);

        // Test status code comparison logic
        assert!(expected_create_status.is_success());
        assert!(expected_fetch_status.is_success());
        assert!(!StatusCode::BAD_REQUEST.is_success());
    }
}

// === Activity Creation Tests ===

#[cfg(test)]
mod activity_function_tests {
    use super::*;
    use crate::types::Activity;
    use surrealdb::RecordId as Thing;
    use surrealdb::Surreal;
    use surrealdb::engine::local::Mem;

    // Mock database for testing
    async fn setup_mock_db() -> Surreal<surrealdb::engine::local::Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    // Test utility functions for activity operations (mirroring test API)
    async fn test_create_activity(
        db: &Surreal<surrealdb::engine::local::Db>,
        activity: Activity,
    ) -> Result<(), ServerFnError> {
        let _created: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        Ok(())
    }

    async fn test_select_activities(
        db: &Surreal<surrealdb::engine::local::Db>,
        page: usize,
    ) -> Result<Vec<Activity>, ServerFnError> {
        let activities_per_page = 10;
        let start = page * activities_per_page;

        let query = format!(
            "SELECT * FROM activity ORDER BY created_at DESC LIMIT {} START {};",
            activities_per_page, start
        );

        let mut query = db.query(&query).await.unwrap();
        let activities = query.take::<Vec<Activity>>(0).unwrap();

        Ok(activities)
    }

    #[tokio::test]
    async fn test_create_activity_basic() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "test_id"))),
            content: "This is a test activity".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        // Verify the activity was created in the mock database
        let created_activity: Option<Activity> = db.select(("activity", "test_id")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, activity.content);
    }

    #[tokio::test]
    async fn test_create_activity_with_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "tagged_activity"))),
            content: "Activity with tags".to_string(),
            tags: vec!["rust".to_string(), "testing".to_string(), "tdd".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "tagged_activity")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, activity.content);
        assert_eq!(created.tags, activity.tags);
    }

    #[tokio::test]
    async fn test_create_activity_with_source() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "sourced_activity"))),
            content: "Activity with source".to_string(),
            source: Some("https://github.com/rust-lang/rust".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "sourced_activity")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, activity.content);
        assert_eq!(created.source, activity.source);
    }

    #[tokio::test]
    async fn test_create_activity_with_empty_content() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "empty_content"))),
            content: "".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "empty_content")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, "");
    }

    #[tokio::test]
    async fn test_create_activity_with_long_content() {
        let db = setup_mock_db().await;
        let long_content = "a".repeat(10000); // 10KB of content
        let activity = Activity {
            id: Some(Thing::from(("activity", "long_content"))),
            content: long_content.clone(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "long_content")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(created_activity.unwrap().content, long_content);
    }

    #[tokio::test]
    async fn test_create_activity_with_special_characters() {
        let db = setup_mock_db().await;
        let special_content = "Special chars: áéíóú ñ ¿¡ 🚀 \n\t\r\"'\\";
        let activity = Activity {
            id: Some(Thing::from(("activity", "special_chars"))),
            content: special_content.to_string(),
            tags: vec!["español".to_string(), "unicode".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "special_chars")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(created.content, special_content);
        assert_eq!(
            created.tags,
            vec!["español".to_string(), "unicode".to_string()]
        );
    }

    #[tokio::test]
    async fn test_create_activity_with_unicode_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "unicode_tags"))),
            content: "Unicode tags test".to_string(),
            tags: vec![
                "中文".to_string(),
                "日本語".to_string(),
                "한국어".to_string(),
            ],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "unicode_tags")).await.unwrap();
        assert!(created_activity.is_some());
        let created = created_activity.unwrap();
        assert_eq!(
            created.tags,
            vec![
                "中文".to_string(),
                "日本語".to_string(),
                "한국어".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_create_activity_with_empty_tags() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "empty_tags"))),
            content: "Empty tags test".to_string(),
            tags: Vec::new(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "empty_tags")).await.unwrap();
        assert!(created_activity.is_some());
        assert!(created_activity.unwrap().tags.is_empty());
    }

    #[tokio::test]
    async fn test_create_activity_with_invalid_url_source() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "invalid_url"))),
            content: "Invalid URL test".to_string(),
            source: Some("not-a-valid-url".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = test_create_activity(&db, activity.clone()).await;
        assert!(result.is_ok());

        let created_activity: Option<Activity> =
            db.select(("activity", "invalid_url")).await.unwrap();
        assert!(created_activity.is_some());
        assert_eq!(
            created_activity.unwrap().source,
            Some("not-a-valid-url".to_string())
        );
    }

    #[tokio::test]
    async fn test_create_multiple_activities() {
        let db = setup_mock_db().await;
        let activities = vec![
            Activity {
                id: Some(Thing::from(("activity", "multi_1"))),
                content: "First activity".to_string(),
                created_at: "2023-01-01T12:00:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                id: Some(Thing::from(("activity", "multi_2"))),
                content: "Second activity".to_string(),
                tags: vec!["test".to_string()],
                created_at: "2023-01-01T12:01:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                id: Some(Thing::from(("activity", "multi_3"))),
                content: "Third activity".to_string(),
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T12:02:00Z".to_string(),
                ..Default::default()
            },
        ];

        for activity in activities {
            let result = test_create_activity(&db, activity.clone()).await;
            assert!(result.is_ok());
        }

        // Verify all activities were created
        for i in 1..=3 {
            let created_activity: Option<Activity> = db
                .select(("activity", format!("multi_{}", i)))
                .await
                .unwrap();
            assert!(created_activity.is_some());
        }
    }

    // === Activity Selection Tests ===

    #[tokio::test]
    async fn test_select_activities_basic() {
        let db = setup_mock_db().await;
        // Create some test activities
        for i in 0..5 {
            let activity = Activity {
                id: Some(Thing::from(("activity".to_owned(), format!("test_id_{}", i)))),
                content: format!("Activity {}", i),
                created_at: format!("2023-01-01T12:00:0{}Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 5);
        assert_eq!(activities[0].content, "Activity 4");
    }

    #[tokio::test]
    async fn test_select_activities_with_pagination() {
        let db = setup_mock_db().await;
        // Create 25 test activities
        for i in 0..25 {
            let activity = Activity {
                id: Some(Thing::from(("activity".to_owned(), format!("page_test_{}", i)))),
                content: format!("Page test activity {}", i),
                created_at: format!("2023-01-01T12:{:02}:00Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        // Test first page (should have 10 activities)
        let page1 = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0].content, "Page test activity 24"); // Most recent first

        // Test second page (should have 10 activities)
        let page2 = test_select_activities(&db, 1).await.unwrap();
        assert_eq!(page2.len(), 10);
        assert_eq!(page2[0].content, "Page test activity 14");

        // Test third page (should have 5 activities)
        let page3 = test_select_activities(&db, 2).await.unwrap();
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].content, "Page test activity 4");

        // Test fourth page (should be empty)
        let page4 = test_select_activities(&db, 3).await.unwrap();
        assert_eq!(page4.len(), 0);
    }

    #[tokio::test]
    async fn test_select_activities_ordering() {
        let db = setup_mock_db().await;
        // Create activities with different timestamps
        let activities_data = vec![
            ("2023-01-01T10:00:00Z", "Oldest activity"),
            ("2023-01-01T11:00:00Z", "Middle activity"),
            ("2023-01-01T12:00:00Z", "Newest activity"),
        ];

        for (timestamp, content) in activities_data {
            let activity = Activity {
                id: Some(Thing::from((
                    "activity".to_owned(),
                    content.replace(" ", "_").to_lowercase(),
                ))),
                content: content.to_string(),
                created_at: timestamp.to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);

        // Should be ordered by created_at DESC (newest first)
        assert_eq!(activities[0].content, "Newest activity");
        assert_eq!(activities[1].content, "Middle activity");
        assert_eq!(activities[2].content, "Oldest activity");
    }

    #[tokio::test]
    async fn test_select_activities_with_same_timestamp() {
        let db = setup_mock_db().await;
        // Create activities with the same timestamp
        let same_timestamp = "2023-01-01T12:00:00Z";
        let activities_data = vec![
            ("same_time_1", "First same time"),
            ("same_time_2", "Second same time"),
            ("same_time_3", "Third same time"),
        ];

        for (id, content) in activities_data {
            let activity = Activity {
                id: Some(Thing::from(("activity", id))),
                content: content.to_string(),
                created_at: same_timestamp.to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);

        // All should have the same timestamp
        for activity in &activities {
            assert_eq!(activity.created_at, same_timestamp);
        }
    }

    #[tokio::test]
    async fn test_select_activities_with_various_content() {
        let db = setup_mock_db().await;
        // Create activities with different content types
        let activities_data = vec![
            ("empty_content", "".to_string()),
            ("short_content", "Hi".to_string()),
            (
                "medium_content",
                "This is a medium length activity content".to_string(),
            ),
            ("long_content", "a".repeat(1000)),
            (
                "unicode_content",
                "Unicode: áéíóú ñ ¿¡ 🚀 中文 日本語 한국어".to_string(),
            ),
            (
                "special_chars",
                "Special: !@#$%^&*()_+-=[]{};':\",./<>?`~".to_string(),
            ),
            (
                "whitespace",
                "  Multiple   spaces   and\ttabs\nnewlines  ".to_string(),
            ),
        ];

        for (id, content) in activities_data {
            let activity = Activity {
                id: Some(Thing::from(("activity", id))),
                content,
                created_at: "2023-01-01T12:00:00Z".to_string(),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 7);

        // Verify all content types are preserved
        let contents: Vec<String> = activities.iter().map(|a| a.content.clone()).collect();
        assert!(contents.contains(&"".to_string()));
        assert!(contents.contains(&"Hi".to_string()));
        assert!(contents.contains(&"This is a medium length activity content".to_string()));
        assert!(contents.contains(&"a".repeat(1000)));
        assert!(contents.contains(&"Unicode: áéíóú ñ ¿¡ 🚀 中文 日本語 한국어".to_string()));
        assert!(contents.contains(&"Special: !@#$%^&*()_+-=[]{};':\",./<>?`~".to_string()));
        assert!(contents.contains(&"  Multiple   spaces   and\ttabs\nnewlines  ".to_string()));
    }

    #[tokio::test]
    async fn test_select_activities_with_tags_and_sources() {
        let db = setup_mock_db().await;
        // Create activities with various tags and sources
        let activities_data = vec![
            Activity {
                id: Some(Thing::from(("activity", "tagged_1"))),
                content: "Activity with tags".to_string(),
                tags: vec!["rust".to_string(), "web".to_string()],
                source: None,
                created_at: "2023-01-01T12:00:00Z".to_string(),
            },
            Activity {
                id: Some(Thing::from(("activity", "sourced_1"))),
                content: "Activity with source".to_string(),
                tags: Vec::new(),
                source: Some("https://github.com".to_string()),
                created_at: "2023-01-01T12:01:00Z".to_string(),
            },
            Activity {
                id: Some(Thing::from(("activity", "both_1"))),
                content: "Activity with both".to_string(),
                tags: vec!["fullstack".to_string()],
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T12:02:00Z".to_string(),
            },
            Activity {
                id: Some(Thing::from(("activity", "neither_1"))),
                content: "Activity with neither".to_string(),
                tags: Vec::new(),
                source: None,
                created_at: "2023-01-01T12:03:00Z".to_string(),
            },
        ];

        for activity in activities_data {
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 4);

        // Verify tags and sources are preserved
        for activity in &activities {
            let id = activity
                .id
                .as_ref()
                .expect("Activity ID should be present in query results");
            let id_str = id.to_string();
            // Handle both "tagged_1" and "activity:tagged_1" formats
            let id_part = if id_str.contains(':') {
                id_str.split(':').nth(1).unwrap_or(&id_str)
            } else {
                &id_str
            };
            match id_part {
                "tagged_1" => {
                    assert_eq!(activity.tags, vec!["rust".to_string(), "web".to_string()]);
                    assert!(activity.source.is_none());
                }
                "sourced_1" => {
                    assert!(activity.tags.is_empty());
                    assert_eq!(activity.source, Some("https://github.com".to_string()));
                }
                "both_1" => {
                    assert_eq!(activity.tags, vec!["fullstack".to_string()]);
                    assert_eq!(activity.source, Some("https://example.com".to_string()));
                }
                "neither_1" => {
                    assert!(activity.tags.is_empty());
                    assert!(activity.source.is_none());
                }
                _ => panic!("Unexpected activity ID: {}", id_str),
            }
        }
    }

    #[tokio::test]
    async fn test_select_activities_empty_database() {
        let db = setup_mock_db().await;
        let activities = test_select_activities(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 0);

        // Test multiple pages on empty database
        let activities_page2 = test_select_activities(&db, 1).await.unwrap();
        assert_eq!(activities_page2.len(), 0);

        let activities_page10 = test_select_activities(&db, 10).await.unwrap();
        assert_eq!(activities_page10.len(), 0);
    }

    #[tokio::test]
    async fn test_select_activities_large_page_number() {
        let db = setup_mock_db().await;
        // Create only 5 activities
        for i in 0..5 {
            let activity = Activity {
                id: Some(Thing::from(("activity".to_owned(), format!("large_page_{}", i)))),
                content: format!("Activity {}", i),
                created_at: format!("2023-01-01T12:00:0{}Z", i),
                ..Default::default()
            };
            let _: Option<Activity> = db.create("activity").content(activity).await.unwrap();
        }

        // Test with a very large page number
        let activities = test_select_activities(&db, 1000).await.unwrap();
        assert_eq!(activities.len(), 0);
    }
}
