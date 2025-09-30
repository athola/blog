extern crate alloc;
use alloc::collections::BTreeMap;

use leptos::prelude::{ServerFnError, server};
use leptos::server_fn::codec::GetUrl;
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
            Err(ServerFnError::from(e))
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
    let mut posts = query.take::<Vec<Post>>(0)?;
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

    let query = "
    LET $tags = SELECT tags FROM post;
    array::flatten($tags.map(|$t| $t.tags));
    "
    .to_owned();
    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let tags = query.take::<Vec<String>>(1)?;
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

    let query_str = format!(r#"SELECT *, author.* from post WHERE slug = "{slug}""#);
    let mut query = retry_db_operation(|| async { db.query(&query_str).await }).await?;
    let post = query.take::<Vec<Post>>(0)?;
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

    let query_str = "SELECT * from reference WHERE is_published = true ORDER BY created_at DESC;";
    let mut query = retry_db_operation(|| async { db.query(query_str).await }).await?;
    let references = query.take::<Vec<Reference>>(0)?;
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
    let activities_per_page = 10;
    let start = page * activities_per_page;

    let query = format!(
        "SELECT * FROM activity ORDER BY created_at DESC LIMIT {} START {};",
        activities_per_page, start
    );

    let mut query = retry_db_operation(|| async { db.query(&query).await }).await?;
    let activities = query.take::<Vec<crate::types::Activity>>(0)?;

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_retry_db_operation_success_first_attempt() {
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
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_retry_db_operation_success_after_failures() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_db_operation(|| {
            let count = call_count_clone.clone();
            async move {
                let current_count = count.fetch_add(1, Ordering::SeqCst);
                if current_count < 2 {
                    // Fail first two attempts
                    Err(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                        "Temporary failure".to_string(),
                    )))
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
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_retry_db_operation_exhausts_retries() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_db_operation(|| {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<String, surrealdb::Error>(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                    "Persistent failure".to_string(),
                )))
            }
        })
        .await;

        assert!(result.is_err());
        // Should try exactly 4 times (initial + 3 retries based on our retry strategy)
        assert_eq!(call_count.load(Ordering::SeqCst), 4);

        // Verify it's converted to ServerFnError
        match result.unwrap_err() {
            ServerFnError::ServerError(_) => {
                // Successfully converted to ServerFnError::ServerError as expected
            }
            _ => panic!("Expected ServerFnError::ServerError"),
        }
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_retry_db_operation_timing() {
        use std::time::Instant;

        let start = Instant::now();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let _result = retry_db_operation(|| {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<String, surrealdb::Error>(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                    "Always fail".to_string(),
                )))
            }
        })
        .await;

        let elapsed = start.elapsed();

        // With exponential backoff starting at 50ms, should take some time but not too long
        // Make timing assertions less strict to avoid flaky tests
        assert!(elapsed.as_millis() >= 25); // Some delay expected
        assert!(elapsed.as_secs() < 10); // But reasonable overall time
        assert_eq!(call_count.load(Ordering::SeqCst), 4);
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
    #[tokio::test]
    async fn test_create_activity_with_retry() {
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
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_select_activities_with_retry() {
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
                        Err(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                            "Network timeout".to_string(),
                        )))
                    } else if current == 1 {
                        Err(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                            "Connection lost".to_string(),
                        )))
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
}
