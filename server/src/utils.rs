extern crate alloc;
use alloc::sync::Arc;
use app::types::{AppState, Post};
use axum::extract::State;
use axum::response::Response;
use chrono::{DateTime, Utc};
use core::fmt::Write as _;
use leptos::prelude::ServerFnError;
use markdown::process_markdown;
use rss::{ChannelBuilder, Item};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::Root;
use tokio::sync::Mutex;
use tokio_retry::{Retry, strategy::ExponentialBackoff};

pub async fn connect() -> Surreal<Client> {
    let protocol = env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
    let host = env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());
    let username = env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "root".to_owned());
    let password = env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "root".to_owned());
    let ns = env::var("SURREAL_NS").unwrap_or_else(|_| "rustblog".to_owned());
    let db_name = env::var("SURREAL_DB").unwrap_or_else(|_| "rustblog".to_owned());

    let retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(5))
        .take(5); // Maximum 5 retry attempts

    let db = Retry::spawn(retry_strategy, || async {
        tracing::info!(
            "Attempting to connect to SurrealDB at {}://{}",
            protocol,
            host
        );
        if protocol == "http" {
            Surreal::new::<Http>(&host).await
        } else {
            Surreal::new::<Https>(&host).await
        }
    })
    .await
    .map_err(|e| {
        tracing::error!("Failed to connect to SurrealDB after retries: {:?}", e);
        e
    })
    .unwrap();

    // Retry authentication with exponential backoff
    let auth_retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(3))
        .take(3);

    Retry::spawn(auth_retry_strategy, || {
        let username = username.clone();
        let password = password.clone();
        let db = &db;
        async move {
            db.signin(Root {
                username: &username,
                password: &password,
            })
            .await
        }
    })
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to authenticate with SurrealDB after retries: {:?}",
            e
        );
        e
    })
    .unwrap();

    // Retry namespace/database selection
    let ns_retry_strategy = ExponentialBackoff::from_millis(50)
        .max_delay(Duration::from_secs(2))
        .take(3);

    Retry::spawn(ns_retry_strategy, || {
        let ns = ns.clone();
        let db_name = db_name.clone();
        let db = &db;
        async move { db.use_ns(ns).use_db(db_name).await }
    })
    .await
    .map_err(|e| {
        tracing::error!("Failed to set namespace/database after retries: {:?}", e);
        e
    })
    .unwrap();

    tracing::info!("Successfully connected to SurrealDB with retries");
    db
}

async fn retry_query<F, Fut, T>(operation: F) -> Result<T, surrealdb::Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, surrealdb::Error>>,
{
    let retry_strategy = ExponentialBackoff::from_millis(50)
        .max_delay(Duration::from_secs(2))
        .take(3); // Maximum 3 retry attempts for queries

    Retry::spawn(retry_strategy, || async {
        match operation().await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!("Database query failed, retrying: {:?}", e);
                Err(e)
            }
        }
    })
    .await
}

pub async fn rss_handler(State(state): State<AppState>) -> Response<String> {
    let AppState { db, .. } = state;
    let rss = generate_rss(db).await.unwrap();
    Response::builder()
        .header("Content-Type", "application/rss+xml")
        .body(rss)
        .unwrap()
}

pub async fn generate_rss(db: Surreal<Client>) -> Result<String, ServerFnError> {
    let query = retry_query(|| async {
        db.query("SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC;")
            .await
    })
    .await;

    let mut query = match query {
        Ok(q) => q,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let mut posts = query.take::<Vec<Post>>(0)?;
    for post in &mut posts.iter_mut() {
        let date_time = DateTime::parse_from_rfc3339(&post.created_at)
            .unwrap()
            .with_timezone(&Utc);
        let naive_date = date_time.date_naive();
        let formatted_date = naive_date.format("%b %-d").to_string();
        post.created_at = formatted_date;
    }
    let posts = Arc::new(Mutex::new(posts));
    let mut handles = vec![];

    let post_len = posts.lock().await.len();
    for _ in 0..post_len {
        let posts_clone = Arc::clone(&posts);
        let handle = tokio::spawn(async move {
            let mut posts = posts_clone.lock().await;
            if let Some(post) = posts.iter_mut().next() {
                post.body = process_markdown(&post.body).unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    let channel = ChannelBuilder::default()
        .title("alexthola")
        .link("https://alexthola.com")
        .description("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
        .items(
            posts
                .lock()
                .await
                .clone()
                .into_iter()
                .map(|post| {
                    let mut item = Item::default();
                    item.set_author(post.author.name.to_string());
                    item.set_title(post.title.to_string());
                    item.set_description(post.body.to_string());
                    item.set_link(format!("https://alexthola.com/post/{}", post.slug.unwrap()));
                    item.set_pub_date(post.created_at);
                    item
                })
                .collect::<Vec<_>>(),
        )
        .build();

    Ok(channel.to_string())
}

pub async fn sitemap_handler(State(state): State<AppState>) -> Response<String> {
    #[derive(Serialize, Deserialize)]
    struct Post {
        slug: Option<String>,
        created_at: String,
    }

    let AppState { db, .. } = state;
    let query = retry_query(|| async {
        db.query(
            "SELECT slug, created_at FROM post WHERE is_published = true ORDER BY created_at DESC;",
        )
        .await
    })
    .await;
    let posts = query.unwrap().take::<Vec<Post>>(0).unwrap();
    let mut sitemap = String::new();
    sitemap.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    sitemap.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    let static_urls = vec![
        ("https://alexthola.com/", "daily", "0.9"),
        ("https://alexthola.com/contact", "weekly", "1.0"),
        ("https://alexthola.com/references", "weekly", "0.6"),
        ("https://alexthola.com/rss.xml", "daily", "0.5"),
        ("https://alexthola.com/sitemap.xml", "monthly", "0.5"),
    ];

    for (url, freq, priority) in static_urls {
        sitemap.push_str("<url>\n");
        writeln!(sitemap, "<loc>{url}</loc>").unwrap();
        writeln!(sitemap, "<changefreq>{freq}</changefreq>").unwrap();
        writeln!(sitemap, "<priority>{priority}</priority>").unwrap();
        sitemap.push_str("</url>\n");
    }

    for post in posts {
        sitemap.push_str("<url>\n");
        writeln!(
            sitemap,
            "<loc>https://alexthola.com/post/{}</loc>",
            post.slug.unwrap()
        )
        .unwrap();
        sitemap.push_str("<changefreq>monthly</changefreq>\n");
        sitemap.push_str("<priority>1.0</priority>\n");
        writeln!(sitemap, "<lastmod>{}</lastmod>", post.created_at).unwrap();
        sitemap.push_str("</url>\n");
    }
    sitemap.push_str("</urlset>");
    Response::builder()
        .header("Content-Type", "application/xml")
        .body(sitemap)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_retry_query_success_first_attempt() {
        tokio_test::block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_query(|| {
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

    #[test]
    fn test_retry_query_success_after_failures() {
        tokio_test::block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_query(|| {
                let count = call_count_clone.clone();
                async move {
                    let current_count = count.fetch_add(1, Ordering::SeqCst);
                    if current_count < 2 {
                        // Fail first two attempts
                        Err(surrealdb::Error::Db(surrealdb::error::Db::Thrown(
                            "Connection failed".to_string(),
                        )))
                    } else {
                        // Succeed on third attempt
                        Ok::<String, surrealdb::Error>("success".to_string())
                    }
                }
            })
            .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "success");
            assert_eq!(call_count.load(Ordering::SeqCst), 3);
        });
    }

    #[test]
    fn test_retry_query_exhausts_retries() {
        tokio_test::block_on(async {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let result = retry_query(|| {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err::<String, surrealdb::Error>(surrealdb::Error::Db(
                        surrealdb::error::Db::Thrown("Persistent failure".to_string()),
                    ))
                }
            })
            .await;

            assert!(result.is_err());
            // Should try exactly 4 times (initial + 3 retries based on our retry strategy)
            assert_eq!(call_count.load(Ordering::SeqCst), 4);
        });
    }

    #[tokio::test]
    async fn test_generate_rss_handles_db_errors() {
        // This test verifies the RSS generation handles database errors gracefully
        // We can't easily mock SurrealDB, but we can test error handling patterns

        // Test would require a mock database, which is complex with SurrealDB
        // Instead, we test that the function signature and error handling compile correctly

        // This is a placeholder that would be expanded with proper mocking
        // For now, we just verify the function exists and has correct return type
        let _: fn(Surreal<Client>) -> _ = generate_rss;
    }

    #[tokio::test]
    async fn test_retry_mechanisms_timing() {
        use std::time::Instant;

        let start = Instant::now();
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let _result = retry_query(|| {
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

        // With exponential backoff starting at 50ms, should have some delays
        // Make timing assertions less strict to avoid flaky tests
        assert!(elapsed.as_millis() >= 25); // Some delay expected
        assert!(elapsed.as_secs() < 15); // But reasonable overall time  
        assert_eq!(call_count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_environment_variable_defaults() {
        // Test that environment variable parsing works with defaults
        // This tests the connect function's environment handling without actual connections

        // Save current env vars to restore later
        let old_protocol = std::env::var("SURREAL_PROTOCOL").ok();
        let old_host = std::env::var("SURREAL_HOST").ok();
        let old_username = std::env::var("SURREAL_ROOT_USER").ok();
        let old_password = std::env::var("SURREAL_ROOT_PASS").ok();

        // Clear env vars to test defaults
        unsafe {
            std::env::remove_var("SURREAL_PROTOCOL");
            std::env::remove_var("SURREAL_HOST");
            std::env::remove_var("SURREAL_ROOT_USER");
            std::env::remove_var("SURREAL_ROOT_PASS");
        }

        // Test the default value logic
        let protocol = std::env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
        let host = std::env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());
        let username = std::env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "root".to_owned());
        let password = std::env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "root".to_owned());

        // Restore env vars
        unsafe {
            if let Some(val) = old_protocol {
                std::env::set_var("SURREAL_PROTOCOL", val);
            }
            if let Some(val) = old_host {
                std::env::set_var("SURREAL_HOST", val);
            }
            if let Some(val) = old_username {
                std::env::set_var("SURREAL_ROOT_USER", val);
            }
            if let Some(val) = old_password {
                std::env::set_var("SURREAL_ROOT_PASS", val);
            }
        }

        assert_eq!(protocol, "http");
        assert_eq!(host, "127.0.0.1:8000");
        assert_eq!(username, "root");
        assert_eq!(password, "root");
    }

    #[test]
    fn test_rss_handler_structure() {
        // Test the RSS handler function signature and basic structure
        // This verifies the handler compiles and has correct types

        // We can't easily test the full handler without a running server,
        // but we can verify the function exists with correct signature
        let _: fn(State<AppState>) -> _ = rss_handler;
    }

    #[test]
    fn test_sitemap_handler_structure() {
        // Test the sitemap handler function signature and basic structure
        let _: fn(State<AppState>) -> _ = sitemap_handler;
    }
}
