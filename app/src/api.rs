//! This module provides server functions for the blog application's API,
//! facilitating interactions with the SurrealDB database.
//!
//! It includes functions for fetching posts, managing tags, handling contact form
//! submissions, retrieving references, and managing activity streams.
//! The module also contains various helper functions for query building,
//! data serialization/deserialization, and error handling.
//!
//! All database operations are wrapped with a retry mechanism to enhance resilience.

// Suppresses the `leptos::server_fn::error::NoCustomError` deprecation, used
// pervasively here via `ServerFnError::<NoCustomError>::ServerError(..)` — the
// idiomatic error type for leptos 0.8 server functions. server_fn 0.9 removes
// the `WrappedServerError` variant; drop this allow during the leptos 0.9
// upgrade and migrate to the new custom-error API.
#![allow(deprecated)]

extern crate alloc;
use alloc::collections::BTreeMap;
use leptos::prelude::{ServerFnError, server};
use leptos::server_fn::codec::GetUrl;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use leptos::server_fn::error::NoCustomError;
#[cfg(feature = "ssr")]
use shared_utils::{
    RetryConfig, is_valid_email, is_valid_slug, is_valid_tag, retry_async, sanitize_html,
};

#[cfg(feature = "ssr")]
use crate::types::Activity;
use crate::types::{Post, Reference};
#[cfg(feature = "ssr")]
use surrealdb::RecordId;

#[cfg(any(feature = "ssr", test))]
const ACTIVITIES_PER_PAGE: usize = 10;

/// Maximum slug length accepted before a database lookup (matches the
/// canonical limit enforced by `server::validation::validate_slug`).
#[cfg(feature = "ssr")]
const MAX_SLUG_LEN: usize = 200;

/// Maximum tag length accepted before a database lookup.
#[cfg(feature = "ssr")]
const MAX_TAG_LEN: usize = 100;

/// Fetches a list of blog posts from the database.
///
/// Posts can be filtered by a list of tags. If no tags are provided,
/// all published posts are returned. Posts are ordered by creation date (descending).
/// The `created_at` timestamp is also formatted for display.
///
/// # Arguments
///
/// * `tags` - An optional `Vec<String>` to filter posts by.
///
/// # Returns
///
/// A `Result` containing a `Vec<Post>` on success, or a `ServerFnError` on failure.
#[server(endpoint = "/posts")]
pub async fn select_posts(
    #[server(default)] tags: Vec<String>,
) -> Result<Vec<Post>, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    // Use parameterized queries to prevent SQL injection
    let mut posts: Vec<Post> = if tags.is_empty() {
        let mut query = retry_async("select_posts", RetryConfig::default(), || async {
            db.query(
                "SELECT *, author.* FROM post WHERE is_published = true ORDER BY created_at DESC",
            )
            .await
        })
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;
        query
            .take(0)
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?
    } else {
        // Validate all tags as secondary defense layer
        for tag in &tags {
            if !is_valid_tag(tag, MAX_TAG_LEN) {
                return Err(ServerFnError::<NoCustomError>::ServerError(format!(
                    "Invalid tag format: '{}'",
                    tag.chars().take(50).collect::<String>()
                )));
            }
        }
        // Use parameterized query with array binding
        let mut query = retry_async("select_posts", RetryConfig::default(), || {
            let tags = tags.clone();
            async move {
                db.query("SELECT *, author.* FROM post WHERE tags CONTAINSANY $tags ORDER BY created_at DESC")
                    .bind(("tags", tags))
                    .await
            }
        })
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;
        query
            .take(0)
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?
    };
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

/// Fetches all unique tags from published posts and returns them with their counts.
///
/// The tags are returned in a `BTreeMap` where the key is the tag name and the
/// value is the count of posts associated with that tag.
///
/// # Returns
///
/// A `Result` containing a `BTreeMap<String, usize>` on success, or a `ServerFnError` on failure.
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
    let mut query = retry_async("select_tags", RetryConfig::default(), || async {
        db.query(&query).await
    })
    .await
    .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;
    let tags = query
        .take::<Vec<String>>(1)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?;
    let mut tag_map = BTreeMap::<String, usize>::new();
    for tag in tags {
        *tag_map.entry(tag).or_insert(0) += 1;
    }

    Ok(tag_map)
}

/// Fetches a single blog post by its slug.
///
/// The post's content is processed from Markdown to HTML, and the `created_at`
/// timestamp is formatted for display.
///
/// # Arguments
///
/// * `slug` - A `String` representing the unique slug of the post.
///
/// # Returns
///
/// A `Result` containing a `Post` on success, or a `ServerFnError` if the post
/// is not found or a database error occurs.
#[server(endpoint = "/post")]
pub async fn select_post(slug: String) -> Result<Post, ServerFnError> {
    use crate::types::AppState;
    use chrono::{DateTime, Utc};
    use leptos::prelude::expect_context;
    use markdown::process_markdown;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    // Validate slug format as secondary defense layer
    if !is_valid_slug(&slug, MAX_SLUG_LEN) {
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Invalid slug format: '{}'",
            slug.chars().take(50).collect::<String>()
        )));
    }

    // Use parameterized query to prevent SQL injection
    let mut query = retry_async("select_post", RetryConfig::default(), || {
        let slug = slug.clone();
        async move {
            db.query("SELECT *, author.* FROM post WHERE slug = $slug")
                .bind(("slug", slug))
                .await
        }
    })
    .await
    .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;
    let post: Vec<Post> = query
        .take(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?;
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

/// Increments the `total_views` count for a specific post.
///
/// # Arguments
///
/// * `id` - A `String` representing the unique ID of the post.
///
/// # Returns
///
/// A `Result` indicating success (`()`) or a `ServerFnError` on failure.
#[server(endpoint = "/increment_views")]
pub async fn increment_views(id: String) -> Result<(), ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    // Validate id format as secondary defense layer
    if !is_valid_slug(&id, MAX_SLUG_LEN) {
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Invalid post id format: '{}'",
            id.chars().take(50).collect::<String>()
        )));
    }

    // Use parameterized query to prevent SQL injection
    // SurrealDB record ID syntax: type:id (e.g., post:abc123)
    retry_async("increment_views", RetryConfig::default(), || {
        let id = id.clone();
        async move {
            db.query("UPDATE type::thing('post', $id) SET total_views = total_views + 1")
                .bind(("id", id))
                .await
        }
    })
    .await
    .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;

    Ok(())
}

/// Contact form submission data.
///
/// # Security
/// - The `website` field acts as a honeypot to detect automated bots.
///   Legitimate users (with browsers) won't see or fill this field.
/// - If `website` is not empty, the submission is rejected as likely bot traffic.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContactRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    /// Honeypot field - should be empty for legitimate submissions.
    /// Bots often auto-fill all fields, so we check this server-side.
    #[serde(default)]
    pub website: Option<String>,
}

/// Validates a contact-form email address, returning its normalized (trimmed,
/// lowercased) form.
///
/// Delegates to the canonical [`is_valid_email`] rule so the contact form — the
/// highest-risk user input point — is validated no more weakly than
/// `server::validation::validate_email`.
#[cfg(feature = "ssr")]
fn validate_contact_email(input: &str) -> Result<String, String> {
    if !is_valid_email(input) {
        return Err("Invalid email format".to_string());
    }
    Ok(input.trim().to_lowercase())
}

/// Sanitizes a required contact-form field: trims, HTML-escapes, and enforces
/// non-empty plus a maximum length. `field` labels the field in error messages.
#[cfg(feature = "ssr")]
fn sanitize_required_field(value: &str, field: &str, max_len: usize) -> Result<String, String> {
    let sanitized = sanitize_html(value.trim());
    if sanitized.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    if sanitized.len() > max_len {
        return Err(format!("{field} too long (max {max_len} characters)"));
    }
    Ok(sanitized)
}

/// Handles contact form submissions by sending an email.
///
/// This function constructs an email from the `ContactRequest` data and attempts
/// to send it using an SMTP transport. It includes an exponential backoff retry
/// mechanism to enhance reliability in case of transient email sending failures.
///
/// SMTP configuration (host, user, password) is loaded from environment variables.
///
/// # Security
///
/// - All user inputs are sanitized to prevent XSS in email clients
/// - Email format is validated before processing
/// - Honeypot field is checked for bot detection
///
/// # Arguments
///
/// * `data` - A `ContactRequest` struct containing the sender's details and message.
///
/// # Returns
///
/// A `Result` indicating success (`()`) or a `ServerFnError` on failure (e.g.,
/// SMTP configuration issues, email sending failures after retries).
#[server(endpoint = "/contact")]
pub async fn contact(data: ContactRequest) -> Result<(), ServerFnError> {
    use lettre::{
        AsyncSmtpTransport, AsyncTransport as _, Message, Tokio1Executor,
        message::header::ContentType, transport::smtp::authentication::Credentials,
    };
    use std::env;

    // Anti-bot protection: Validate honeypot field.
    // Legitimate users won't see or fill this field, but bots often auto-fill all fields.
    if let Some(ref website) = data.website
        && !website.is_empty()
    {
        tracing::warn!("Contact form rejected: honeypot field was filled (likely bot)");
        // Return success to avoid tipping off bots - they think it worked
        return Ok(());
    }

    // Validate and sanitize all user inputs to prevent XSS in email clients
    let validated_email =
        validate_contact_email(&data.email).map_err(ServerFnError::<NoCustomError>::ServerError)?;

    let sanitized_name = sanitize_required_field(&data.name, "Name", 100)
        .map_err(ServerFnError::<NoCustomError>::ServerError)?;
    let sanitized_subject = sanitize_required_field(&data.subject, "Subject", 200)
        .map_err(ServerFnError::<NoCustomError>::ServerError)?;
    let sanitized_message = sanitize_required_field(&data.message, "Message", 5000)
        .map_err(ServerFnError::<NoCustomError>::ServerError)?;

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&env::var("SMTP_HOST")?)?
        .credentials(Credentials::new(
            env::var("SMTP_USER")?,
            env::var("SMTP_PASSWORD")?,
        ))
        .build::<Tokio1Executor>();

    // Build email body using sanitized inputs
    let email_body = format!(
        "From: {} ({})\n\nMessage:\n{}",
        sanitized_name, validated_email, sanitized_message
    );

    let email = Message::builder()
        .from(env::var("SMTP_USER")?.parse()?)
        .to(env::var("SMTP_USER")?.parse()?)
        .subject(format!("{} - {}", validated_email, sanitized_subject))
        .header(ContentType::TEXT_PLAIN)
        .body(email_body)?;

    // Send the email with exponential backoff (200ms base, 10s cap, 3 attempts),
    // sharing the same retry/logging path as every database operation.
    match retry_async("contact_email", RetryConfig::new(200, 10, 3), || async {
        mailer.send(email.clone()).await
    })
    .await
    {
        Ok(response) => {
            tracing::info!("Email sent successfully: {response:?}");
            Ok(())
        }
        Err(email_err) => {
            tracing::error!("Failed to send email after retries: {email_err:?}");
            Err(ServerFnError::from(email_err))
        }
    }
}

/// Fetches a list of published references from the database.
///
/// References are ordered by creation date (descending).
///
/// # Returns
///
/// A `Result` containing a `Vec<Reference>` on success, or a `ServerFnError` on failure.
#[server(endpoint = "/references")]
pub async fn select_references() -> Result<Vec<Reference>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let query_str = "SELECT * from reference WHERE is_published = true ORDER BY created_at DESC;";
    let mut query = retry_async("select_references", RetryConfig::default(), || async {
        db.query(query_str).await
    })
    .await
    .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;
    let references = query
        .take::<Vec<Reference>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?;
    Ok(references)
}

/// Represents pagination parameters for fetching data.
#[derive(Deserialize)]
pub struct Pagination {
    pub page: usize,
}

/// Creates a new activity record in the database.
///
/// This server function uses SurrealDB's native API to create the activity record.
/// It includes a retry mechanism for database operations to handle transient errors.
///
/// # Arguments
///
/// * `activity` - The `Activity` struct containing the data for the new record.
///
/// # Returns
///
/// A `Result` indicating success (`()`) or a `ServerFnError` on failure
/// (e.g., database error, serialization failure).
///
/// Inserts an `Activity` record using SurrealDB's native API.
///
/// Generic over the connection type so the same insert logic serves both the
/// production `Surreal<Client>` path and the in-memory `Surreal<Any>` used in
/// tests. When `activity.id` is set it is converted to a `RecordId` and used as
/// the record key; otherwise the record is created in the `activity` table.
#[cfg(feature = "ssr")]
async fn insert_activity<C: surrealdb::Connection>(
    db: &surrealdb::Surreal<C>,
    activity: Activity,
) -> Result<(), Box<surrealdb::Error>> {
    let _: Option<Activity> = if let Some(id) = activity.id.clone() {
        // The id is specified in the create call, so drop it from the content.
        let mut content_activity = activity.clone();
        content_activity.id = None;

        let table: &str = &id.tb;
        let id_str = id.id.to_string();
        let record_id = RecordId::from((table, id_str.as_str()));
        db.create::<Option<Activity>>(record_id)
            .content(content_activity)
            .await
            .map_err(Box::new)?
    } else {
        db.create::<Option<Activity>>("activity")
            .content(activity)
            .await
            .map_err(Box::new)?
    };
    Ok(())
}

/// Fetches one page of activity records, newest first. Shared by the
/// `select_activities` server function and its test harness so the query string
/// and bindings live in exactly one place (mirrors `insert_activity`).
#[cfg(any(feature = "ssr", test))]
async fn query_activities_page<C: surrealdb::Connection>(
    db: &surrealdb::Surreal<C>,
    page: usize,
) -> Result<Vec<Activity>, Box<surrealdb::Error>> {
    let start = page * ACTIVITIES_PER_PAGE;
    db.query("SELECT * FROM activity ORDER BY created_at DESC LIMIT $limit START $start")
        .bind(("limit", ACTIVITIES_PER_PAGE))
        .bind(("start", start))
        .await
        .map_err(Box::new)?
        .take(0)
        .map_err(Box::new)
}

#[server(prefix = "/api/activities", endpoint = "create")]
pub async fn create_activity(
    api_key: String,
    activity: crate::types::Activity,
) -> Result<(), ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    // Verify the caller-supplied API key against the server-side secret.
    let expected_key = std::env::var("ACTIVITY_API_KEY").unwrap_or_default();
    if expected_key.is_empty() || api_key != expected_key {
        return Err(ServerFnError::<NoCustomError>::ServerError(
            "Forbidden: invalid or missing API key".to_string(),
        ));
    }

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let create_result: Result<(), String> =
        retry_async("create_activity", RetryConfig::default(), || async {
            insert_activity(db, activity.clone())
                .await
                .map_err(|e| e.to_string())
        })
        .await;

    create_result
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;

    Ok(())
}

/// Selects activity records from the database with pagination.
///
/// This server function constructs a SurrealQL `SELECT` query to fetch a page
/// of activity records, ordered by creation date. It includes a retry mechanism
/// for database operations and deserializes the raw SurrealDB `Value`s into
/// `Activity` structs.
///
/// # Arguments
///
/// * `page` - The 0-indexed page number of activities to retrieve.
///
/// # Returns
///
/// A `Result` containing a `Vec<Activity>` on success, or a `ServerFnError` on failure
/// (e.g., database error, deserialization failure).
#[server(prefix = "/api", endpoint = "activities", input = GetUrl)]
pub async fn select_activities(
    #[server(default)] page: usize,
) -> Result<Vec<crate::types::Activity>, ServerFnError> {
    use crate::types::AppState;
    use leptos::prelude::expect_context;

    let AppState { db, .. } = expect_context::<AppState>();
    let db = db.as_ref();

    let activities: Vec<Activity> =
        retry_async("select_activities", RetryConfig::default(), || async {
            query_activities_page(db, page).await.map_err(|e| {
                ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}"))
            })
        })
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Database error: {e}")))?;

    Ok(activities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;
    #[cfg(feature = "ssr")]
    use surrealdb::Surreal;
    #[cfg(feature = "ssr")]
    use surrealdb::engine::any::Any;
    #[cfg(feature = "ssr")]
    use surrealdb::sql::Thing;
    #[cfg(feature = "ssr")]
    use tokio_test::block_on;

    // Slug/tag/email validation rules are unit-tested in the `shared_utils`
    // crate, the canonical home for those helpers.

    /// Verifies the default state of a `ContactRequest`.
    #[test]
    fn test_contact_request_default() {
        let request = ContactRequest::default();
        assert_eq!(request.name, "");
        assert_eq!(request.email, "");
        assert_eq!(request.subject, "");
        assert_eq!(request.message, "");
        assert_eq!(request.website, None);
    }

    /// Confirms that `ContactRequest` serializes and deserializes correctly.
    #[test]
    fn test_contact_request_serialization() {
        let request = ContactRequest {
            name: "Test Name".to_string(),
            email: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            message: "Test Message".to_string(),
            website: None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: ContactRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.name, deserialized.name);
        assert_eq!(request.email, deserialized.email);
        assert_eq!(request.subject, deserialized.subject);
        assert_eq!(request.message, deserialized.message);
    }
    #[test]
    /// Confirms the default values of the `Activity` struct.
    fn test_activity_default() {
        let activity = Activity::default();
        assert_eq!(activity.content, "");
        assert_eq!(activity.tags, Vec::<String>::new());
        assert_eq!(activity.source, None);
        assert_eq!(activity.created_at, "");
    }
    #[test]
    /// Verifies `Activity` struct serialization and deserialization.
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
    /// Verifies the `Pagination` struct initializes correctly.
    fn test_pagination_struct() {
        let pagination = Pagination { page: 1 };
        assert_eq!(pagination.page, 1);
    }
    #[cfg(feature = "ssr")]
    #[test]
    /// Tests the `create_activity` function's existence and serialization capabilities.
    fn test_create_activity_basics() {
        block_on(async {
            let activity = Activity {
                content: "Test activity content".to_string(),
                tags: vec!["test".to_string()],
                source: Some("https://test.com".to_string()),
                ..Default::default()
            };
            let serialized = serde_json::to_string(&activity).unwrap();
            assert!(!serialized.is_empty());
            assert!(serialized.contains("Test activity content"));
        });
    }
    #[test]
    /// Verifies `Activity` struct's compatibility with expected JSON structure.
    fn test_activity_json_compatibility() {
        let activity_json = serde_json::json!({
            "content": "This is a test activity",
            "tags": ["test", "rust"],
            "source": "https://example.com"
        });
        let activity: Activity = serde_json::from_value(activity_json).unwrap();

        assert_eq!(activity.content, "This is a test activity");
        assert_eq!(activity.tags, vec!["test", "rust"]);
        assert_eq!(activity.source, Some("https://example.com".to_string()));
    }
    #[test]
    /// Tests various valid activity creation scenarios, including empty tags and multiple tags.
    fn test_activity_creation_scenarios() {
        let test_cases = vec![
            Activity {
                content: "Valid activity".to_string(),
                tags: vec!["test".to_string()],
                source: Some("https://example.com".to_string()),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                content: "Activity with no tags".to_string(),
                tags: vec![],
                source: None,
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
            Activity {
                content: "Multi-tag activity".to_string(),
                tags: vec!["rust".to_string(), "web".to_string(), "blog".to_string()],
                source: Some("https://blog.example.com".to_string()),
                created_at: "2023-01-01T00:00:00Z".to_string(),
                ..Default::default()
            },
        ];

        for activity in test_cases {
            let serialized = serde_json::to_string(&activity).unwrap();
            let deserialized: Activity = serde_json::from_str(&serialized).unwrap();
            assert_eq!(activity, deserialized);
        }
    }
    #[test]
    /// Tests error handling for invalid activity data deserialization.
    fn test_activity_error_deserialization() {
        let invalid_activity_json = serde_json::json!({
            "content": 123,
            "tags": "not-an-array",
            "source": null
        });
        let result: Result<Activity, _> = serde_json::from_value(invalid_activity_json);
        assert!(
            result.is_err(),
            "Invalid activity data should fail deserialization"
        );
    }
    #[test]
    /// Verifies the expected JSON response format for activities.
    fn test_activity_json_response_format() {
        let activity_response = serde_json::json!([
            {
                "content": "This is a test activity",
                "tags": ["test", "rust"],
                "source": "https://example.com",
                "created_at": "2023-01-01T00:00:00Z"
            }
        ]);
        let activities: Vec<Activity> = serde_json::from_value(activity_response).unwrap();
        assert_eq!(activities.len(), 1);
        assert_eq!(activities[0].content, "This is a test activity");
        assert_eq!(activities[0].tags, vec!["test", "rust"]);
        assert_eq!(
            activities[0].source,
            Some("https://example.com".to_string())
        );
    }
    // === Activity Integration Tests (Mock Database) ===

    /// Sets up a mock SurrealDB instance for testing.
    async fn setup_mock_db() -> Surreal<Any> {
        let db: Surreal<Any> = Surreal::init();
        db.connect("memory").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    /// Helper to create an activity in the mock database. Delegates to the same
    /// `insert_activity` used by the production `create_activity` server function.
    async fn create_activity_in_db(
        db: &Surreal<Any>,
        activity: Activity,
    ) -> Result<(), ServerFnError> {
        insert_activity(db, activity)
            .await
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Create error: {e}")))
    }

    /// Helper to select activities from the mock database. Delegates to the same
    /// `query_activities_page` used by the production `select_activities` server
    /// function so the query and bindings are exercised by these tests.
    async fn select_activities_from_db(
        db: &Surreal<Any>,
        page: usize,
    ) -> Result<Vec<Activity>, ServerFnError> {
        query_activities_page(db, page)
            .await
            .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))
    }

    /// Helper to fetch a specific activity by its ID from the mock database.
    async fn fetch_activity_by_id_from_db(db: &Surreal<Any>, id: &str) -> Option<Activity> {
        // Use SurrealDB's native API - it handles special characters properly
        (db.select::<Option<Activity>>(("activity", id)).await).unwrap_or_default()
    }

    /// Tests basic activity creation with a mock database.
    #[tokio::test]
    async fn test_create_activity_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "test_id"))),
            content: "This is a test activity".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok(), "create_activity failed: {:?}", result.err());

        let created_activity = fetch_activity_by_id_from_db(&db, "test_id")
            .await
            .expect("activity:test_id should exist");
        assert_eq!(created_activity.content, "This is a test activity");
    }

    /// Tests activity creation with tags.
    #[tokio::test]
    async fn test_create_activity_with_tags_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "tagged_activity"))),
            content: "Activity with tags".to_string(),
            tags: vec!["rust".to_string(), "testing".to_string(), "tdd".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok(), "create_activity failed: {:?}", result.err());

        let created = fetch_activity_by_id_from_db(&db, "tagged_activity")
            .await
            .expect("activity:tagged_activity missing");
        assert_eq!(created.content, "Activity with tags");
        assert_eq!(created.tags, vec!["rust", "testing", "tdd"]);
    }

    /// Tests activity creation with a source URL.
    #[tokio::test]
    async fn test_create_activity_with_source_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "sourced_activity"))),
            content: "Activity with source".to_string(),
            source: Some("https://github.com/rust-lang/rust".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created = fetch_activity_by_id_from_db(&db, "sourced_activity")
            .await
            .expect("activity:sourced_activity missing");
        assert_eq!(created.content, "Activity with source");
        assert_eq!(
            created.source,
            Some("https://github.com/rust-lang/rust".to_string())
        );
    }

    /// Tests activity creation with empty content.
    #[tokio::test]
    async fn test_create_activity_empty_content_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "empty_content"))),
            content: "".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created = fetch_activity_by_id_from_db(&db, "empty_content")
            .await
            .expect("activity:empty_content missing");
        assert_eq!(created.content, "");
    }

    /// Tests activity creation with long content.
    #[tokio::test]
    async fn test_create_activity_long_content_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "long_content"))),
            content: "a".repeat(10000),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let fetched = fetch_activity_by_id_from_db(&db, "long_content")
            .await
            .expect("activity:long_content missing");
        assert_eq!(fetched.content.len(), 10000);
        assert!(fetched.content.chars().all(|c| c == 'a'));
    }

    /// Tests activity creation with special characters in content and tags.
    #[tokio::test]
    async fn test_create_activity_special_chars_mock_db() {
        let db = setup_mock_db().await;
        let special_content = "Special chars: áéíóú ñ ¿¡ 🚀 \n\t\r\"'\\";
        let activity = Activity {
            id: Some(Thing::from(("activity", "special_chars"))),
            content: special_content.to_string(),
            tags: vec!["español".to_string(), "unicode".to_string()],
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created = fetch_activity_by_id_from_db(&db, "special_chars")
            .await
            .expect("activity:special_chars missing");
        assert_eq!(created.content, special_content);
        assert_eq!(
            created.tags,
            vec!["español".to_string(), "unicode".to_string()]
        );
    }

    /// Tests activity creation with Unicode tags.
    #[tokio::test]
    async fn test_create_activity_unicode_tags_mock_db() {
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

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created = fetch_activity_by_id_from_db(&db, "unicode_tags")
            .await
            .expect("activity:unicode_tags missing");
        assert_eq!(
            created.tags,
            vec![
                "中文".to_string(),
                "日本語".to_string(),
                "한국어".to_string()
            ]
        );
    }

    /// Tests activity creation with an empty tag list.
    #[tokio::test]
    async fn test_create_activity_empty_tags_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "empty_tags"))),
            content: "Empty tags test".to_string(),
            tags: Vec::new(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created_activity = fetch_activity_by_id_from_db(&db, "empty_tags")
            .await
            .expect("activity:empty_tags missing");
        assert!(created_activity.tags.is_empty());
    }

    /// Tests activity creation with an invalid URL as a source.
    #[tokio::test]
    async fn test_create_activity_invalid_source_url_mock_db() {
        let db = setup_mock_db().await;
        let activity = Activity {
            id: Some(Thing::from(("activity", "invalid_url"))),
            content: "Invalid URL test".to_string(),
            source: Some("not-a-valid-url".to_string()),
            created_at: "2023-01-01T12:00:00Z".to_string(),
            ..Default::default()
        };

        let result = create_activity_in_db(&db, activity).await;
        assert!(result.is_ok());

        let created = fetch_activity_by_id_from_db(&db, "invalid_url")
            .await
            .expect("activity:invalid_url missing");
        assert_eq!(created.source, Some("not-a-valid-url".to_string()));
    }

    /// Tests creation of multiple activities.
    #[tokio::test]
    async fn test_create_multiple_activities_mock_db() {
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
            let result = create_activity_in_db(&db, activity).await;
            assert!(result.is_ok());
        }

        for i in 1..=3 {
            let key = format!("multi_{i}");
            let created_activity = fetch_activity_by_id_from_db(&db, &key).await;
            assert!(created_activity.is_some());
        }
    }

    // === Activity Selection Tests ===

    /// Tests basic activity selection from a mock database.
    #[tokio::test]
    async fn test_select_activities_basic_mock_db() {
        let db = setup_mock_db().await;
        for i in 0..5 {
            let activity = Activity {
                id: Some(Thing::from(("activity", format!("test_id_{i}").as_str()))),
                content: format!("Activity {i}"),
                created_at: format!("2023-01-01T12:00:0{i}Z"),
                ..Default::default()
            };
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 5);
        assert_eq!(activities[0].content, "Activity 4"); // Newest first
    }

    /// Tests activity selection with pagination using a mock database.
    #[tokio::test]
    async fn test_select_activities_pagination_mock_db() {
        let db = setup_mock_db().await;
        for i in 0..25 {
            let activity = Activity {
                id: Some(Thing::from(("activity", format!("page_test_{i}").as_str()))),
                content: format!("Page test activity {i}"),
                created_at: format!("2023-01-01T12:{i:02}:00Z"),
                ..Default::default()
            };
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let page1 = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0].content, "Page test activity 24");

        let page2 = select_activities_from_db(&db, 1).await.unwrap();
        assert_eq!(page2.len(), 10);
        assert_eq!(page2[0].content, "Page test activity 14");

        let page3 = select_activities_from_db(&db, 2).await.unwrap();
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].content, "Page test activity 4");

        let page4 = select_activities_from_db(&db, 3).await.unwrap();
        assert_eq!(page4.len(), 0);
    }

    /// Verifies activity ordering by `created_at` in descending order.
    #[tokio::test]
    async fn test_select_activities_ordering_mock_db() {
        let db = setup_mock_db().await;
        let activities_data = vec![
            ("2023-01-01T10:00:00Z", "Oldest activity"),
            ("2023-01-01T11:00:00Z", "Middle activity"),
            ("2023-01-01T12:00:00Z", "Newest activity"),
        ];

        for (timestamp, content) in activities_data {
            let activity = Activity {
                id: Some(Thing::from((
                    "activity",
                    content.replace(" ", "_").to_lowercase().as_str(),
                ))),
                content: content.to_string(),
                created_at: timestamp.to_string(),
                ..Default::default()
            };
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);
        assert_eq!(activities[0].content, "Newest activity");
        assert_eq!(activities[1].content, "Middle activity");
        assert_eq!(activities[2].content, "Oldest activity");
    }

    /// Tests activity selection when multiple activities share the same timestamp.
    #[tokio::test]
    async fn test_select_activities_same_timestamp_mock_db() {
        let db = setup_mock_db().await;
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
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 3);
        for activity in &activities {
            assert_eq!(activity.created_at, same_timestamp);
        }
    }

    /// Verifies various content types (empty, long, unicode, special chars) are preserved during selection.
    #[tokio::test]
    async fn test_select_activities_various_content_mock_db() {
        let db = setup_mock_db().await;
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
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 7);
        let contents: Vec<&str> = activities.iter().map(|a| a.content.as_str()).collect();
        assert!(contents.contains(&""));
        assert!(contents.contains(&"Hi"));
        assert!(contents.contains(&"This is a medium length activity content"));
        let repeated = "a".repeat(1000);
        assert!(contents.contains(&repeated.as_str()));
        assert!(contents.contains(&"Unicode: áéíóú ñ ¿¡ 🚀 中文 日本語 한국어"));
        assert!(contents.contains(&"Special: !@#$%^&*()_+-=[]{};':\",./<>?`~"));
        assert!(contents.contains(&"  Multiple   spaces   and\ttabs\nnewlines  "));
    }

    /// Tests activity selection with various tags and sources.
    #[tokio::test]
    async fn test_select_activities_tags_and_sources_mock_db() {
        let db = setup_mock_db().await;
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
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 4);
        for activity in &activities {
            let id = activity.id.as_ref().expect("Activity ID should be present");
            let id_part = id.id.to_string();
            match id_part.as_str() {
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
                _ => panic!("Unexpected activity ID: {}", id_part),
            }
        }
    }

    /// Tests activity selection from an empty database.
    #[tokio::test]
    async fn test_select_activities_empty_db() {
        let db = setup_mock_db().await;
        let activities = select_activities_from_db(&db, 0).await.unwrap();
        assert_eq!(activities.len(), 0);

        let activities_page2 = select_activities_from_db(&db, 1).await.unwrap();
        assert_eq!(activities_page2.len(), 0);
    }

    /// Tests activity selection with a page number exceeding available data.
    #[tokio::test]
    async fn test_select_activities_large_page_mock_db() {
        let db = setup_mock_db().await;
        for i in 0..5 {
            let activity = Activity {
                id: Some(Thing::from((
                    "activity",
                    format!("large_page_{i}").as_str(),
                ))),
                content: format!("Activity {i}"),
                created_at: format!("2023-01-01T12:00:0{i}Z"),
                ..Default::default()
            };
            create_activity_in_db(&db, activity).await.unwrap();
        }

        let activities = select_activities_from_db(&db, 1000).await.unwrap();
        assert_eq!(activities.len(), 0);
    }
}
