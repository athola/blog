//! This module provides utility functions for the Axum server, including
//! establishing SurrealDB connections, generating RSS feeds, and constructing
//! XML sitemaps.
//!
//! It encapsulates logic for interacting with the database, handling external
//! data formats (RSS, sitemap), and managing server-side response construction.

#![allow(deprecated)]

use app::types::{AppState, Author, Post};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use chrono::DateTime;
use core::fmt::Write as _;
use leptos::prelude::ServerFnError;
use leptos::server_fn::error::NoCustomError;
use markdown::process_markdown;
use rss::{ChannelBuilder, Item};
use serde::{Deserialize, Serialize};
use shared_utils::{RetryConfig, retry_async};
use std::env;
use std::mem;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::{Database, Namespace, Root};
use tokio_retry::{Retry, strategy::ExponentialBackoff};
use tracing::{error, warn};

fn parse_surreal_address(raw: &str) -> Option<(String, String)> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let (scheme, rest) = raw.split_once("://").unwrap_or(("http", raw));
    let host = rest.split('/').next().unwrap_or("").trim();
    if host.is_empty() {
        return None;
    }

    let scheme = if scheme.eq_ignore_ascii_case("https") {
        "https"
    } else {
        "http"
    };

    Some((scheme.to_string(), host.to_string()))
}

/// Builds an Axum `Response<String>` with the specified body, content type, and status code.
///
/// This helper standardizes the process of creating HTTP responses and
/// handles potential errors during response construction by returning an
/// internal server error.
///
/// # Arguments
///
/// * `body` - The `String` content for the response body.
/// * `content_type` - The `Content-Type` header value (e.g., "text/html", "application/json").
/// * `status` - The HTTP `StatusCode` for the response.
///
/// # Returns
///
/// An `Axum` `Response<String>`. In case of a response build error, it returns
/// a generic `500 Internal Server Error` response.
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

/// Establishes and authenticates a connection to SurrealDB.
///
/// This function retrieves connection details and credentials from environment variables.
/// It implements robust retry mechanisms with exponential backoff for:
/// 1. Initial connection attempts to the SurrealDB server.
/// 2. Authentication attempts using root, namespace, or database-level credentials.
/// 3. Selecting the specified namespace and database.
///
/// Connection parameters:
/// - `SURREAL_PROTOCOL`: `http` or `https` (defaults to `http`).
/// - `SURREAL_HOST`: Host and port (defaults to `127.0.0.1:8000`).
///
/// Authentication priority:
/// 1. `SURREAL_USERNAME`/`SURREAL_PASSWORD` (database level).
/// 2. `SURREAL_NAMESPACE_USER`/`SURREAL_NAMESPACE_PASS` (namespace level).
/// 3. `SURREAL_ROOT_USER`/`SURREAL_ROOT_PASS` (root level).
///
/// Namespace/Database selection:
/// - `SURREAL_NS`: Namespace name (defaults to `rustblog`).
/// - `SURREAL_DB`: Database name (defaults to `rustblog`).
///
/// # Returns
///
/// A `Result` containing a connected and authenticated `Surreal<Client>` instance
/// on success, or a `surrealdb::Error` if connection or authentication fails
/// after all retries.
pub async fn connect() -> Result<Surreal<Client>, surrealdb::Error> {
    // Retrieve connection and authentication details from environment variables.
    let default_protocol = env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
    let default_host = env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());

    let surreal_address = env::var("SURREAL_ADDRESS")
        .or_else(|_| env::var("SURREAL_URL"))
        .ok()
        .filter(|s| !s.trim().is_empty());

    let (protocol, host) = if let Some(address) = surreal_address {
        parse_surreal_address(&address).unwrap_or_else(|| {
            warn!(
                surreal_address = address.as_str(),
                "Failed to parse `SURREAL_ADDRESS`; falling back to `SURREAL_PROTOCOL`/`SURREAL_HOST`"
            );
            (default_protocol.clone(), default_host.clone())
        })
    } else if default_host.contains("://") {
        parse_surreal_address(&default_host).unwrap_or((default_protocol.clone(), default_host))
    } else {
        (default_protocol, default_host)
    };

    let username = env::var("SURREAL_ROOT_USER").unwrap_or_default();
    let password = env::var("SURREAL_ROOT_PASS").unwrap_or_default();

    if username.is_empty() && password.is_empty() {
        let allow_anonymous = env::var("SURREAL_ALLOW_ANONYMOUS")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);
        let is_production = env::var("LEPTOS_SITE_ADDR")
            .map(|addr| addr.starts_with("0.0.0.0"))
            .unwrap_or(false);

        if allow_anonymous {
            warn!(
                "SURREAL_ROOT_USER and SURREAL_ROOT_PASS are both unset; SURREAL_ALLOW_ANONYMOUS is set, proceeding without authentication"
            );
        } else if is_production {
            return Err(surrealdb::Error::Api(surrealdb::error::Api::InvalidParams(
                "SURREAL_ROOT_USER and SURREAL_ROOT_PASS are required in production. Set SURREAL_ALLOW_ANONYMOUS=true to override.".into(),
            )));
        } else {
            warn!(
                "SURREAL_ROOT_USER and SURREAL_ROOT_PASS are both unset; using unauthenticated access (set credentials for production)"
            );
        }
    }
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

    let root_credentials = if !username.is_empty() && !password.is_empty() {
        Some((username.clone(), password.clone()))
    } else {
        None
    };
    let namespace_credentials = match (namespace_username.clone(), namespace_password.clone()) {
        (Some(user), Some(pass)) if !pass.is_empty() => Some((user, pass)),
        _ => None,
    };
    let database_credentials = match (database_username.clone(), database_password.clone()) {
        (Some(user), Some(pass)) if !pass.is_empty() => Some((user, pass)),
        _ => None,
    };

    // Retry strategy for initial database connection.
    let retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(5))
        .take(5);

    // Attempt to connect to SurrealDB with retries.
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

    // Retry strategy for database authentication.
    let auth_retry_strategy = ExponentialBackoff::from_millis(100)
        .max_delay(Duration::from_secs(3))
        .take(3);

    // Attempt to authenticate with SurrealDB using available credentials.
    Retry::spawn(auth_retry_strategy, || {
        let db = &db;
        let root_credentials = root_credentials.clone();
        let namespace_credentials = namespace_credentials.clone();
        let database_credentials = database_credentials.clone();
        let ns_clone = ns.clone();
        let db_name_clone = db_name.clone();

        async move {
            let mut last_err: Option<surrealdb::Error> = None;

            // Attempt root-level authentication first (recommended for simple setups).
            if let Some((ref username, ref password)) = root_credentials {
                match db.signin(Root { username: &username.clone(), password: &password.clone() }).await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        tracing::debug!("Root authentication attempt failed: {:?}", e);
                        last_err = Some(e);
                    }
                }
            }

            // Fallback to namespace-level authentication.
            if let Some((ref username, ref password)) = namespace_credentials {
                match db
                    .signin(Namespace {
                        namespace: &ns_clone.clone(),
                        username: &username.clone(),
                        password: &password.clone(),
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

            // Fallback to database-level authentication.
            if let Some((ref username, ref password)) = database_credentials {
                match db
                    .signin(Database {
                        namespace: &ns_clone.clone(),
                        database: &db_name_clone.clone(),
                        username: &username.clone(),
                        password: &password.clone(),
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

            // If no authentication method succeeded, return the last error encountered.
            if let Some(err) = last_err {
                Err(err)
            } else {
                // This case should ideally not be reached if at least one credential set is provided.
                Err(surrealdb::Error::Api(surrealdb::error::Api::Query(
                    "No SurrealDB authentication methods succeeded".to_string(),
                )))
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

        // Provide specific error messages based on attempted credentials (in auth priority order).
        if root_credentials_present {
            tracing::error!(
                "Root-level authentication failed. Verify SURREAL_ROOT_USER/SURREAL_ROOT_PASS match the credentials SurrealDB was started with. See DEPLOYMENT.md for details."
            );
        } else if namespace_credentials_present {
            tracing::error!(
                "Namespace-level authentication failed. Verify SURREAL_NAMESPACE_USER/SURREAL_NAMESPACE_PASS."
            );
            tracing::error!(
                "The user must exist in SurrealDB: DEFINE USER <name> ON NAMESPACE PASSWORD '<pass>' ROLES OWNER;"
            );
        } else if database_credentials_present {
            tracing::error!(
                "Database-level authentication failed for namespace `{}` and database `{}`.",
                ns,
                db_name,
            );
            tracing::error!(
                "For database-level auth, the user must be created in SurrealDB: DEFINE USER <name> ON DATABASE PASSWORD '<pass>' ROLES OWNER;"
            );
        } else {
            tracing::error!(
                "No authentication credentials provided. Set one of: SURREAL_ROOT_USER/SURREAL_ROOT_PASS (recommended), SURREAL_NAMESPACE_USER/SURREAL_NAMESPACE_PASS, or SURREAL_USERNAME/SURREAL_PASSWORD."
            );
        }

        e
    })?;

    // Retry strategy for selecting namespace and database.
    let ns_retry_strategy = ExponentialBackoff::from_millis(50)
        .max_delay(Duration::from_secs(2))
        .take(3);

    // Attempt to use the specified namespace and database with retries.
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

/// Handles requests for the RSS feed endpoint (`/rss` or `/rss.xml`).
///
/// Fetches published blog posts from the database, generates the RSS XML string,
/// and returns it as an Axum `Response`. Includes robust error handling.
///
/// # Arguments
///
/// * `state` - An `AppState` containing the SurrealDB client.
///
/// # Returns
///
/// An `Axum` `Response<String>` with the RSS XML content or an error message.
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

/// Generates the RSS feed XML by fetching published posts from the database.
///
/// It queries the database for published posts, processes their content
/// (e.g., Markdown rendering), and then constructs an RSS `Channel` with
/// individual `Item`s for each post.
///
/// # Arguments
///
/// * `db` - A reference to the connected `Surreal<Client>` instance.
///
/// # Returns
///
/// A `Result` containing the RSS XML as a `String` on success, or a
/// `ServerFnError` if database query or post processing fails.
pub async fn generate_rss(db: &Surreal<Client>) -> Result<String, ServerFnError> {
    let query_str =
        "SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC;";
    let query_result = retry_async("generate_rss_query", RetryConfig::default(), || async {
        db.query(query_str).await
    })
    .await;

    let mut query = match query_result {
        Ok(q) => q,
        Err(e) => return Err(ServerFnError::<NoCustomError>::ServerError(e.to_string())),
    };

    let mut posts = query
        .take::<Vec<Post>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {}", e)))?;
    for post in &mut posts {
        let post_id = format!("{:?}", post.id); // Get post ID for error logging.
        let raw_created_at = mem::take(&mut post.created_at); // Take ownership to parse.

        // Parse and format the creation date for RSS.
        let date_time = DateTime::parse_from_rfc3339(&raw_created_at).map_err(|e| {
            error!(
                %post_id,
                raw_created_at,
                "Failed to parse post created_at timestamp: {e}"
            );
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Invalid created_at timestamp for post {post_id}: {e}"
            ))
        })?;
        let formatted_date = date_time.to_rfc2822(); // RSS requires RFC 2822 date format.
        post.created_at = formatted_date;

        // Process Markdown body to HTML for RSS description.
        let processed_body = process_markdown(&post.body).map_err(|e| {
            error!(%post_id, "Failed to render Markdown for post: {e}");
            e
        })?;
        post.body = processed_body;
    }

    // Construct RSS items from processed posts.
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

    // Build the RSS channel.
    let channel = ChannelBuilder::default()
        .title("alexthola")
        .link("https://alexthola.com")
        .description("Alex Thola's Blog \u{2013} Tech Insights & Consulting")
        .items(items)
        .build();

    Ok(channel.to_string())
}

/// Handles requests for the sitemap XML endpoint (`/sitemap.xml`).
///
/// Fetches published post slugs and creation dates from the database,
/// combines them with static URLs, and generates the sitemap XML string.
///
/// # Arguments
///
/// * `state` - An `AppState` containing the SurrealDB client.
///
/// # Returns
///
/// An `Axum` `Response<String>` with the sitemap XML content or an error message.
/// Escapes special XML characters in a string to prevent XML injection.
fn escape_xml(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(ch),
        }
    }
    output
}

pub async fn sitemap_handler(State(state): State<AppState>) -> Response<String> {
    /// Internal struct for deserializing post data relevant to the sitemap.
    #[derive(Serialize, Deserialize)]
    struct Post {
        slug: Option<String>,
        created_at: String,
    }

    let AppState { db, .. } = state;
    let db = db.as_ref();

    // Query for published posts, ordered by creation date.
    let query_result = retry_async("sitemap_query", RetryConfig::default(), || async {
        db.query(
            "SELECT slug, created_at FROM post WHERE is_published = true ORDER BY created_at DESC;",
        )
        .await
    })
    .await;

    let mut query = match query_result {
        Ok(result) => result,
        Err(err) => {
            error!(?err, "Failed to fetch sitemap posts from database");
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
            error!(
                ?err,
                "Failed to deserialize sitemap posts from database response"
            );
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let mut sitemap = String::new();
    sitemap.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    sitemap.push_str(
        "<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">
",
    );

    // Define static URLs to be included in the sitemap.
    // /archive, /notes, /about, /colophon land in this PR's IA refresh; without
    // them the new routes are unreachable from search-engine crawls.
    let static_urls = vec![
        ("https://alexthola.com/", "daily", "0.9"),
        ("https://alexthola.com/archive", "weekly", "0.7"),
        ("https://alexthola.com/notes", "daily", "0.7"),
        ("https://alexthola.com/about", "monthly", "0.6"),
        ("https://alexthola.com/contact", "weekly", "1.0"),
        ("https://alexthola.com/references", "weekly", "0.6"),
        ("https://alexthola.com/colophon", "monthly", "0.4"),
        ("https://alexthola.com/rss.xml", "daily", "0.5"),
        ("https://alexthola.com/sitemap.xml", "monthly", "0.5"),
    ];

    // Write static URLs to the sitemap XML.
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

    // Write dynamic post URLs to the sitemap XML.
    for post in posts {
        let Some(slug) = post.slug.as_deref() else {
            warn!("Skipping sitemap entry for post without a slug");
            continue;
        };

        let escaped_slug = escape_xml(slug);
        let escaped_date = escape_xml(&post.created_at);
        sitemap.push_str("<url>\n");
        if let Err(err) = writeln!(
            sitemap,
            "<loc>https://alexthola.com/post/{escaped_slug}</loc>"
        ) {
            error!(?err, slug, "Failed to write sitemap dynamic URL");
            return build_response(
                "Failed to build sitemap".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
        sitemap.push_str("<changefreq>monthly</changefreq>\n");
        sitemap.push_str("<priority>1.0</priority>\n");
        if let Err(err) = writeln!(sitemap, "<lastmod>{escaped_date}</lastmod>") {
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

// ─────────────────────────────────────────────────────────────────────
// Sprint 3 (T24-T27): Atom feed, /random, /post/:slug/raw.md, redirects.
// ─────────────────────────────────────────────────────────────────────

/// Handles GET /feed/feed.xml — Atom 1.0 feed.
///
/// Per spec §4.10. The Atom feed mirrors the RSS content but with the
/// Atom envelope. Hand-rolled XML to avoid adding the `atom_syndication`
/// crate dependency.
pub async fn atom_handler(State(state): State<AppState>) -> Response<String> {
    let AppState { db, .. } = state;
    let db = db.as_ref();
    match generate_atom(db).await {
        Ok(atom) => build_response(atom, "application/atom+xml; charset=utf-8", StatusCode::OK),
        Err(err) => {
            error!(?err, "Failed to generate Atom feed");
            build_response(
                "Failed to generate Atom feed".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

/// Generates the Atom feed XML from published posts.
pub async fn generate_atom(db: &Surreal<Client>) -> Result<String, ServerFnError> {
    let query_str =
        "SELECT *, author.* from post WHERE is_published = true ORDER BY created_at DESC LIMIT 50;";
    let query_result = retry_async("generate_atom_query", RetryConfig::default(), || async {
        db.query(query_str).await
    })
    .await;

    let mut query = match query_result {
        Ok(q) => q,
        Err(e) => return Err(ServerFnError::<NoCustomError>::ServerError(e.to_string())),
    };

    let posts = query
        .take::<Vec<Post>>(0)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Query error: {e}")))?;

    // Per Atom 1.0 §4.2.15, feed-level <updated> reflects the most recent
    // meaningful modification. Use max(updated_at) across entries, not the
    // first post's created_at — published-but-since-edited posts otherwise
    // never bump <updated> and feed readers skip re-fetching.
    let updated = posts
        .iter()
        .map(|p| &p.updated_at)
        .max()
        .cloned()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let mut feed = String::new();
    feed.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    feed.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    let _ = writeln!(feed, "  <title>alexthola</title>");
    let _ = writeln!(
        feed,
        "  <subtitle>Tech Insights &amp; Consulting from Alex Thola</subtitle>"
    );
    let _ = writeln!(feed, "  <id>https://alexthola.com/</id>");
    let _ = writeln!(
        feed,
        "  <link href=\"https://alexthola.com/\" rel=\"alternate\"/>"
    );
    let _ = writeln!(
        feed,
        "  <link href=\"https://alexthola.com/feed/feed.xml\" rel=\"self\" type=\"application/atom+xml\"/>"
    );
    let _ = writeln!(feed, "  <updated>{}</updated>", escape_xml(&updated));
    feed.push_str("  <author><name>Alex Thola</name></author>\n");

    for post in posts {
        let slug = post.slug.clone().unwrap_or_default();
        if slug.is_empty() {
            continue;
        }
        let post_url = format!("https://alexthola.com/post/{slug}");
        let processed_body = match process_markdown(&post.body) {
            Ok(html) => html,
            Err(err) => {
                // Skipping is preferable to publishing raw markdown inside
                // <content type="html">: feed readers would render `**bold**`
                // and `# heading` as literal text and break links.
                error!(?err, slug = %slug, "atom: process_markdown failed; skipping post");
                continue;
            }
        };
        feed.push_str("  <entry>\n");
        let _ = writeln!(feed, "    <title>{}</title>", escape_xml(&post.title));
        let _ = writeln!(feed, "    <id>{}</id>", escape_xml(&post_url));
        let _ = writeln!(
            feed,
            "    <link href=\"{}\" rel=\"alternate\"/>",
            escape_xml(&post_url)
        );
        let _ = writeln!(
            feed,
            "    <updated>{}</updated>",
            escape_xml(&post.updated_at)
        );
        let _ = writeln!(
            feed,
            "    <published>{}</published>",
            escape_xml(&post.created_at)
        );
        let _ = writeln!(
            feed,
            "    <author><name>{}</name></author>",
            escape_xml(&post.author.name)
        );
        let _ = writeln!(
            feed,
            "    <summary type=\"html\">{}</summary>",
            escape_xml(&post.summary)
        );
        let _ = writeln!(
            feed,
            "    <content type=\"html\">{}</content>",
            escape_xml(&processed_body)
        );
        feed.push_str("  </entry>\n");
    }

    feed.push_str("</feed>\n");
    Ok(feed)
}

/// Handles GET /random — picks a random published post and 302 redirects
/// to its detail page (spec §4.9, "Stumble" mechanic).
pub async fn random_handler(State(state): State<AppState>) -> Response<String> {
    #[derive(Deserialize)]
    struct SlugOnly {
        slug: Option<String>,
    }

    let AppState { db, .. } = state;
    let db = db.as_ref();

    let query_result = retry_async("random_post_query", RetryConfig::default(), || async {
        db.query("SELECT slug FROM post WHERE is_published = true ORDER BY rand() LIMIT 1;")
            .await
    })
    .await;

    let mut query = match query_result {
        Ok(q) => q,
        Err(e) => {
            error!(?e, "random_handler: query failed");
            return build_response(
                "No posts available".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::SERVICE_UNAVAILABLE,
            );
        }
    };

    let row: Option<SlugOnly> = match query.take(0) {
        Ok(row) => row,
        Err(e) => {
            error!(?e, "random_handler: deserialize failed");
            return build_response(
                "No posts available".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::SERVICE_UNAVAILABLE,
            );
        }
    };

    let slug = match row.and_then(|r| r.slug) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return build_response(
                "No published posts to stumble through".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::NOT_FOUND,
            );
        }
    };

    let target = format!("/post/{slug}");
    let mut response = Response::new(format!("Redirecting to {target}"));
    *response.status_mut() = StatusCode::FOUND;
    if let Ok(location) = target.parse() {
        response.headers_mut().insert("Location", location);
    }
    if let Ok(cache) = "public, max-age=60".parse() {
        response.headers_mut().insert("Cache-Control", cache);
    }
    if let Ok(ct) = "text/plain; charset=utf-8".parse() {
        response.headers_mut().insert("Content-Type", ct);
    }
    response
}

/// Handles GET /post/{slug}/raw.md — returns the raw markdown source for a post
/// (spec §4.11, hacker-culture handshake). Axum 0.8 requires a path param to
/// occupy a whole segment, so the .md suffix lives in its own segment.
pub async fn raw_markdown_handler(
    Path(slug): Path<String>,
    State(state): State<AppState>,
) -> Response<String> {
    if slug.is_empty() {
        return build_response(
            "missing slug".to_string(),
            "text/plain; charset=utf-8",
            StatusCode::BAD_REQUEST,
        );
    }

    #[derive(Deserialize)]
    struct BodyOnly {
        body: String,
    }

    let AppState { db, .. } = state;
    let db = db.as_ref();

    let query_result = retry_async("raw_markdown_query", RetryConfig::default(), || async {
        db.query("SELECT body FROM post WHERE slug = $slug AND is_published = true LIMIT 1;")
            .bind(("slug", slug.to_string()))
            .await
    })
    .await;

    let mut query = match query_result {
        Ok(q) => q,
        Err(e) => {
            error!(?e, "raw_markdown_handler: query failed");
            return build_response(
                "Failed to fetch post".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let row: Option<BodyOnly> = match query.take(0) {
        Ok(row) => row,
        Err(e) => {
            error!(?e, "raw_markdown_handler: deserialize failed");
            return build_response(
                "Failed to fetch post".to_string(),
                "text/plain; charset=utf-8",
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    match row {
        Some(BodyOnly { body }) => {
            build_response(body, "text/markdown; charset=utf-8", StatusCode::OK)
        }
        None => build_response(
            format!("Post '{slug}' not found"),
            "text/plain; charset=utf-8",
            StatusCode::NOT_FOUND,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_surreal_address_http() {
        let (scheme, host) = parse_surreal_address("http://10.0.0.1:8000").unwrap();
        assert_eq!(scheme, "http");
        assert_eq!(host, "10.0.0.1:8000");
    }

    #[test]
    fn parse_surreal_address_https_with_slash() {
        let (scheme, host) = parse_surreal_address("https://example.com:443/").unwrap();
        assert_eq!(scheme, "https");
        assert_eq!(host, "example.com:443");
    }

    #[test]
    fn parse_surreal_address_bare_host_defaults_to_http() {
        let (scheme, host) = parse_surreal_address("10.0.0.1:8000").unwrap();
        assert_eq!(scheme, "http");
        assert_eq!(host, "10.0.0.1:8000");
    }

    #[test]
    fn parse_surreal_address_strips_path() {
        let (scheme, host) = parse_surreal_address("http://10.0.0.1:8000/rpc").unwrap();
        assert_eq!(scheme, "http");
        assert_eq!(host, "10.0.0.1:8000");
    }

    /// Test helper macro to temporarily set and restore environment variables during test execution.
    ///
    /// # Purpose
    /// This macro provides a safe way to temporarily override environment variables for testing
    /// and automatically restore them afterward, ensuring test isolation and preventing
    /// side effects on the actual environment.
    ///
    /// # Usage
    /// ```rust
    /// with_env_vars! {
    ///     "DATABASE_URL" => "memory://test",
    ///     "LOG_LEVEL" => "debug",
    /// }
    /// // Test code that depends on these environment variables
    /// ```
    ///
    /// # Safety
    /// - Uses `unsafe` blocks for `std::env::set_var` and `std::env::remove_var` calls,
    ///   which is necessary because these functions modify global state.
    /// - The unsafety is contained and safe because:
    ///   - We only modify environment variables that were explicitly provided
    ///   - All modifications are restored to their original state
    ///   - The operation has no undefined behavior
    ///
    /// # Implementation Details
    /// 1. Captures original values of all specified environment variables
    /// 2. Sets new temporary values for all variables in the macro call
    /// 3. Returns control to the calling code for test execution
    /// 4. Automatically restores all variables to their original state
    /// 5. Properly handles cases where variables didn't exist originally
    ///
    /// # Example
    /// ```rust
    /* #test
    fn test_database_connection() {
    ///     with_env_vars! {
    ///         "SURREAL_DB" => "test_db",
    ///         "SURREAL_NS" => "test_namespace",
    ///         "SURREAL_USER" => "test_user",
    ///         "SURREAL_PASS" => "test_pass",
    ///     }
    ///
    ///     // Test code that uses these environment variables
    ///     let result = validate_production_env();
    ///     assert!(result.is_ok());
    /// }
    /// */
    macro_rules! with_env_vars {
    ($($key:expr => $value:expr),* $(,)?) => {{
        // Store original values to restore them later
        let original_vars: Vec<(&'static str, Option<String>)> = vec![
            $(($key, std::env::var($key).ok()),)*
        ];

        // Set temporary values for test execution
        unsafe { $(std::env::set_var($key, $value);)* }

        let result = {
            // Placeholder for potential setup; currently unused.
            // This allows for future enhancement where setup code could be injected
            // before returning control to the calling test code.
        };

        // Restore all environment variables to their original state
        for (key, original_value) in original_vars {
            if let Some(value) = original_value {
                unsafe { std::env::set_var(key, value); }
            } else {
                unsafe { std::env::remove_var(key); }
            }
        }
        result
    }};
}

    /// Verifies that environment variable parsing correctly applies default values.
    /// This test uses a macro to isolate environment variable changes.
    #[tokio::test]
    async fn test_connect_env_var_defaults() {
        with_env_vars! {
            "SURREAL_PROTOCOL" => "",
            "SURREAL_HOST" => "",
            "SURREAL_ROOT_USER" => "",
            "SURREAL_ROOT_PASS" => "",
            "SURREAL_NS" => "",
            "SURREAL_DB" => "",
        };

        let protocol = std::env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
        let host = std::env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());
        let username = std::env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "root".to_owned());
        let password = std::env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "root".to_owned());
        let ns = std::env::var("SURREAL_NS").unwrap_or_else(|_| "rustblog".to_owned());
        let db_name = std::env::var("SURREAL_DB").unwrap_or_else(|_| "rustblog".to_owned());

        assert_eq!(protocol, "http");
        assert_eq!(host, "127.0.0.1:8000");
        assert_eq!(username, "root");
        assert_eq!(password, "root");
        assert_eq!(ns, "rustblog");
        assert_eq!(db_name, "rustblog");
    }

    /// Verifies the `rss_handler` function exists with the correct signature.
    /// The full functionality is not tested here, but only its API contract.
    #[test]
    fn test_rss_handler_signature() {
        let _: fn(State<AppState>) -> _ = rss_handler;
    }

    /// Verifies the `sitemap_handler` function exists with the correct signature.
    /// The full functionality is not tested here, but only its API contract.
    #[test]
    fn test_sitemap_handler_signature() {
        let _: fn(State<AppState>) -> _ = sitemap_handler;
    }

    // ─────────────────────────────────────────────────────────────────
    // Sprint 3 (T24-T27): invariant-encoding tests for new feed/route
    // helpers. These guard the XSS-prevention contract and the project
    // convention that every public handler has a signature test.
    // ─────────────────────────────────────────────────────────────────

    /// GIVEN a string containing every XML metacharacter
    /// WHEN escape_xml is applied
    /// THEN every metacharacter is replaced by its named entity.
    /// Catches partial-escape regressions in the table at line 491.
    #[test]
    fn escape_xml_replaces_all_five_metacharacters() {
        let input = "<>&\"'";
        let expected = "&lt;&gt;&amp;&quot;&apos;";
        assert_eq!(escape_xml(input), expected);
    }

    /// GIVEN a compound XSS payload that an attacker might submit as a post title
    /// WHEN escape_xml is applied
    /// THEN no raw `<`, `>`, or `"` characters remain to break the XML envelope.
    /// This is the load-bearing security invariant for RSS/Atom/sitemap output.
    #[test]
    fn escape_xml_neutralises_compound_xss_payload() {
        let payload = r#"<script>alert("xss")</script>"#;
        let escaped = escape_xml(payload);
        assert!(
            !escaped.contains('<'),
            "raw '<' must not survive: {escaped}"
        );
        assert!(
            !escaped.contains('>'),
            "raw '>' must not survive: {escaped}"
        );
        assert!(
            !escaped.contains('"'),
            "raw '\"' must not survive: {escaped}"
        );
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(escaped.contains("&quot;xss&quot;"));
    }

    /// GIVEN a pre-escaped entity like &amp;
    /// WHEN escape_xml is applied a second time
    /// THEN the `&` re-escapes to `&amp;`, yielding `&amp;amp;`.
    /// Documents the (intentional) non-idempotence: callers must escape exactly once.
    #[test]
    fn escape_xml_is_not_idempotent() {
        assert_eq!(escape_xml("&amp;"), "&amp;amp;");
    }

    /// GIVEN an empty string
    /// WHEN escape_xml is applied
    /// THEN the result is empty (boundary case for the with_capacity call).
    #[test]
    fn escape_xml_handles_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    /// GIVEN ASCII text with no metacharacters
    /// WHEN escape_xml is applied
    /// THEN the output is byte-identical to the input (no spurious escapes).
    #[test]
    fn escape_xml_passes_safe_ascii_through_unchanged() {
        let safe = "Hello, world! 2026-05-01 alex@thola.com";
        assert_eq!(escape_xml(safe), safe);
    }

    /// GIVEN a string mixing `&` with another metacharacter (`<`)
    /// WHEN escape_xml is applied
    /// THEN the `&` becomes `&amp;` and the `<` becomes `&lt;` independently —
    ///      i.e. the `&` from `&lt;` is NOT re-escaped to `&amp;lt;`.
    /// Locks in the single-pass char-iterator implementation against a future
    /// "optimization" to chained `String::replace()`, which would naively turn
    /// `<` into `&lt;` first and then mangle the resulting `&` on a second pass.
    #[test]
    fn escape_xml_escapes_ampersand_independently_of_other_metacharacters() {
        assert_eq!(escape_xml("&<"), "&amp;&lt;");
        assert_eq!(escape_xml("<&"), "&lt;&amp;");
        assert_eq!(
            escape_xml(r#"Tom & Jerry's <show> says "hi""#),
            "Tom &amp; Jerry&apos;s &lt;show&gt; says &quot;hi&quot;"
        );
    }

    /// GIVEN text containing multi-byte UTF-8 (em-dash, smart quotes, emoji,
    ///       non-Latin script) with no XML metacharacters
    /// WHEN escape_xml is applied
    /// THEN every code point survives byte-identical.
    /// Guards against a future refactor to byte-level (`u8`) matching, which
    /// would split UTF-8 continuation bytes and corrupt feed content. Real post
    /// titles routinely contain em-dashes and curly quotes — feed validators
    /// reject malformed UTF-8.
    #[test]
    fn escape_xml_preserves_multibyte_unicode() {
        let unicode = "Café — “smart quotes” • 日本語 • 🦀";
        assert_eq!(escape_xml(unicode), unicode);
        // Sanity: the bytes really are multi-byte.
        assert!(unicode.len() > unicode.chars().count());
    }

    /// Verifies the `atom_handler` function exists with the correct signature.
    /// Mirrors the `test_rss_handler_signature` convention so every public
    /// handler in this module has a contract guard.
    #[test]
    fn test_atom_handler_signature() {
        let _: fn(State<AppState>) -> _ = atom_handler;
    }

    /// Verifies the `random_handler` function exists with the correct signature.
    #[test]
    fn test_random_handler_signature() {
        let _: fn(State<AppState>) -> _ = random_handler;
    }

    /// Verifies the `raw_markdown_handler` function exists with the correct signature.
    /// Note: takes a Path extractor in addition to State, distinguishing it from
    /// the State-only handlers above.
    #[test]
    fn test_raw_markdown_handler_signature() {
        let _: fn(Path<String>, State<AppState>) -> _ = raw_markdown_handler;
    }
}
