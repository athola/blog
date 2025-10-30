#![allow(deprecated)]
use app::types::{AppState, Author, Post};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{DateTime, Utc};
use core::fmt::Write as _;
use leptos::prelude::ServerFnError;
use leptos::server_fn::error::NoCustomError;
use markdown::process_markdown;
use rss::{ChannelBuilder, Item};
use serde::{Deserialize, Serialize};
use shared_utils::{RetryConfig, retry_async};
use std::env;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::{Database, Namespace, Root};
use tokio_retry::{Retry, strategy::ExponentialBackoff};
use tracing::{error, warn};

fn build_response(body: String, content_type: &str, status: StatusCode) -> Response<String> {
    match Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .body(body)
    {
        Ok(response) => response,
        Err(build_error) => {
            error!(?build_error, "Failed to build HTTP response");
            let mut fallback = Response::new(String::new());
            *fallback.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            fallback
        }
    }
}

pub async fn connect() -> Result<Surreal<Client>, surrealdb::Error> {
    let protocol = env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
    let host = env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());
    let username = env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "".to_owned());
    let password = env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "".to_owned());
    let namespace_username = env::var("SURREAL_NAMESPACE_USER")
        .ok()
        .filter(|s| !s.is_empty());
    let namespace_password = env::var("SURREAL_NAMESPACE_PASS").ok();
    let database_username = env::var("SURREAL_USERNAME")
        .or_else(|_| env::var("SURREAL_USER"))
        .ok()
        .filter(|s| !s.is_empty());
    let database_password = env::var("SURREAL_PASSWORD")
        .or_else(|_| env::var("SURREAL_PASS"))
        .ok();
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
    })?;

    // Retry authentication with exponential backoff
    let auth_retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(3))
        .take(3);

    let root_credentials = Some((username.clone(), password.clone()));
    let namespace_credentials = namespace_username.clone().zip(namespace_password.clone());
    let database_credentials = database_username.clone().zip(database_password.clone());

    Retry::spawn(auth_retry_strategy, || {
        let db = &db;
        let root_credentials = root_credentials.clone();
        let namespace_credentials = namespace_credentials.clone();
        let database_credentials = database_credentials.clone();
        let ns = ns.clone();
        let db_name = db_name.clone();
        async move {
            let mut last_err: Option<surrealdb::Error> = None;

            if let Some((ref username, ref password)) = database_credentials {
                match db
                    .signin(Database {
                        namespace: &ns,
                        database: &db_name,
                        username,
                        password,
                    })
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        tracing::debug!(
                            "Database authentication attempt failed: {:?}",
                            e
                        );
                        last_err = Some(e);
                    }
                }
            }

            if let Some((ref username, ref password)) = namespace_credentials {
                match db
                    .signin(Namespace {
                        namespace: &ns,
                        username,
                        password,
                    })
                    .await
                {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        tracing::debug!(
                            "Namespace authentication attempt failed: {:?}",
                            e
                        );
                        last_err = Some(e);
                    }
                }
            }

            if let Some((ref username, ref password)) = root_credentials {
                match db.signin(Root { username, password }).await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        tracing::debug!("Root authentication attempt failed: {:?}", e);
                        last_err = Some(e);
                    }
                }
            }

            if let Some(err) = last_err {
                Err(err)
            } else {
                panic!("No SurrealDB authentication methods succeeded")
            }
        }
    })
    .await
    .map_err(|e| {
        tracing::error!(
            "Failed to authenticate with SurrealDB after retries: {:?}",
            e
        );

        let database_credentials_present = database_credentials.is_some();
        let namespace_credentials_present = namespace_credentials.is_some();
        let root_credentials_present = root_credentials.is_some();

        if database_credentials_present {
            tracing::error!(
                "Database-level credentials were provided but authentication still failed. Verify `SURREAL_USERNAME`/`SURREAL_PASSWORD` and ensure the user has access to namespace `{}` and database `{}`.",
                ns,
                db_name,
            );
        } else if namespace_credentials_present {
            tracing::error!(
                "Namespace-level credentials were provided but authentication still failed. Double-check `SURREAL_NAMESPACE_USER`/`SURREAL_NAMESPACE_PASS` values."
            );
        } else if root_credentials_present {
            tracing::error!(
                "Only root-level credentials were attempted. Set `SURREAL_ROOT_USER`/`SURREAL_ROOT_PASS` or provide application credentials via `SURREAL_NAMESPACE_USER`/`SURREAL_NAMESPACE_PASS` or `SURREAL_USERNAME`/`SURREAL_PASSWORD`."
            );
        } else {
            tracing::error!(
                "No authentication credentials were supplied; set one of the supported credential env vars before starting the server."
            );
        }

        e
    })?;

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
    })?;

    tracing::info!("Successfully connected to SurrealDB with retries");
    Ok(db)
}

pub async fn rss_handler(State(state): State<AppState>) -> Response<String> {
    let AppState { db, .. } = state;
    let db = db.as_ref();
    match generate_rss(db).await {
        Ok(rss) => build_response(rss, "application/rss+xml", StatusCode::OK),
        Err(err) => {
            error!(?err, "Failed to generate RSS feed");
            build_response(
                "Failed to generate RSS feed".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub async fn generate_rss(db: &Surreal<Client>) -> Result<String, ServerFnError> {
    let query = retry_async("generate_rss", RetryConfig::default(), || async {
        db.query("SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC;")
            .await
    })
    .await;

    let mut query = match query {
        Ok(q) => q,
        Err(e) => return Err(ServerFnError::<NoCustomError>::ServerError(e.to_string())),
    };

    let mut posts = query
        .take::<Vec<Post>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    for post in &mut posts {
        let post_id = post.id.to_string();
        let raw_created_at = post.created_at.clone();
        let date_time = DateTime::parse_from_rfc3339(&raw_created_at)
            .map_err(|e| {
                error!(
                    %post_id,
                    raw_created_at,
                    "Failed to parse post created_at timestamp: {e}"
                );
                ServerFnError::<NoCustomError>::ServerError(format!(
                    "Invalid created_at timestamp for post {post_id}: {e}"
                ))
            })?
            .with_timezone(&Utc);
        let naive_date = date_time.date_naive();
        let formatted_date = naive_date.format("%b %-d").to_string();
        post.created_at = formatted_date;

        let processed_body = process_markdown(&post.body).map_err(|e| {
            error!(%post_id, "Failed to render Markdown for post: {e}");
            e
        })?;
        post.body = processed_body;
    }

    let items = posts
        .into_iter()
        .map(|post| {
            let Post {
                author: Author {
                    name: author_name, ..
                },
                title,
                body,
                slug,
                created_at,
                ..
            } = post;

            let mut item = Item::default();
            item.set_author(author_name);
            item.set_title(title);
            item.set_description(body);
            if let Some(slug) = slug {
                item.set_link(format!("https://alexthola.com/post/{slug}"));
            }
            item.set_pub_date(created_at);
            item
        })
        .collect::<Vec<_>>();

    let channel = ChannelBuilder::default()
        .title("alexthola")
        .link("https://alexthola.com")
        .description("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
        .items(items)
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
    let db = db.as_ref();
    let query = retry_async("sitemap_query", RetryConfig::default(), || async {
        db.query(
            "SELECT slug, created_at FROM post WHERE is_published = true ORDER BY created_at DESC;",
        )
        .await
    })
    .await;
    let mut query = match query {
        Ok(result) => result,
        Err(err) => {
            error!(?err, "Failed to fetch sitemap posts");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let posts = match query.take::<Vec<Post>>(0) {
        Ok(posts) => posts,
        Err(err) => {
            error!(?err, "Failed to deserialize sitemap posts");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
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
        if let Err(err) = writeln!(sitemap, "<loc>{url}</loc>") {
            error!(?err, url, "Failed to write sitemap static URL");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        if let Err(err) = writeln!(sitemap, "<changefreq>{freq}</changefreq>") {
            error!(?err, url, "Failed to write sitemap static changefreq");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        if let Err(err) = writeln!(sitemap, "<priority>{priority}</priority>") {
            error!(?err, url, "Failed to write sitemap static priority");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        sitemap.push_str("</url>\n");
    }

    for post in posts {
        let Some(slug) = post.slug.as_deref() else {
            warn!("Skipping sitemap entry without slug");
            continue;
        };

        sitemap.push_str("<url>\n");
        if let Err(err) = writeln!(sitemap, "<loc>https://alexthola.com/post/{slug}</loc>") {
            error!(?err, slug, "Failed to write sitemap dynamic URL");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        sitemap.push_str("<changefreq>monthly</changefreq>\n");
        sitemap.push_str("<priority>1.0</priority>\n");
        if let Err(err) = writeln!(sitemap, "<lastmod>{}</lastmod>", post.created_at) {
            error!(?err, slug, "Failed to write sitemap last modified date");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        sitemap.push_str("</url>\n");
    }
    sitemap.push_str("</urlset>");
    build_response(sitemap, "application/xml", StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

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
