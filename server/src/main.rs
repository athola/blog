//! This module is the main entry point for the Axum server, serving the Leptos
//! application.
//!
//! It handles server initialization, configuration loading, database connections,
//! routing, middleware setup (security headers, rate limiting, compression),
//! and static asset serving.

mod redirect;
mod security;
mod utils;
pub mod validation;

use app::{component, shell, types::AppState};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::{Json, Redirect, Response},
    routing::{any, get},
};
use dotenvy::dotenv;
use leptos::logging;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes as _, generate_route_list};
use leptos_config::get_configuration;
use redirect::redirect_www;
use security::{
    RateLimiter, SecurityConfig, security_headers_with_config, validate_production_env,
    validate_smtp_config,
};
use serde_json::json;

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;
use tower::ServiceExt as _;
use tower_http::compression::predicate::{NotForContentType, SizeAbove};
use tower_http::compression::{CompressionLayer, Predicate as _};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utils::{connect, rss_handler, sitemap_handler};

fn choose_site_addr(
    config_addr: SocketAddr,
    leptos_site_addr_env: Option<&str>,
    port_env: Option<&str>,
) -> SocketAddr {
    if let Some(raw) = leptos_site_addr_env
        .map(str::trim)
        .filter(|s| !s.is_empty())
        && let Ok(addr) = raw.parse::<SocketAddr>()
    {
        return addr;
    }

    if let Some(raw) = port_env.map(str::trim).filter(|s| !s.is_empty())
        && let Ok(port) = raw.parse::<u16>()
    {
        return SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    }

    config_addr
}

/// Handles the `/health` endpoint, returning a JSON response indicating the server's status.
///
/// This endpoint is used for health checks by load balancers and monitoring systems.
/// It includes basic service information like status, timestamp, service name, and version.
///
/// # Returns
///
/// A `Result` containing a JSON response with health information, or an `Axum`
/// `StatusCode` on failure (though unlikely for this handler).
async fn health_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "blog-api",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// Main entry point for the Axum server.
///
/// This asynchronous function initializes the server, sets up logging, loads
/// configuration, connects to the SurrealDB database, configures Axum routes
/// and middleware, and starts listening for incoming requests.
///
/// The server serves the Leptos full-stack application, handling both server-side
/// rendering (SSR) and API requests.
#[tokio::main]
async fn main() {
    // Configure tracing (logging) based on debug assertions.
    let tracing_level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_max_level(tracing_level)
        .init();

    // Load environment variables from a `.env` file if present.
    // In production, env vars are typically provided externally by the host platform,
    // so a missing `.env` should not cause errors.
    let _ = dotenv();

    // Validate essential environment variables.
    // In production mode (RUST_ENV=production), validation failures are fatal.
    // In development mode, we warn but continue to allow easier local testing.
    let is_production =
        std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "production";

    if let Err(errors) = validate_production_env() {
        for error in &errors {
            logging::error!("Environment validation error: {}", error);
        }

        if is_production {
            logging::error!(
                "FATAL: {} environment validation error(s) in production mode. \
                 Set required environment variables or use RUST_ENV=development for local testing.",
                errors.len()
            );
            std::process::exit(1);
        } else {
            logging::warn!(
                "Continuing despite {} environment validation error(s) in development mode.",
                errors.len()
            );
        }
    }

    // Log SMTP configuration status for debugging email issues.
    let smtp_status = validate_smtp_config();
    if smtp_status.is_valid() {
        logging::log!("SMTP configuration: Valid");
    } else if smtp_status.is_configured {
        logging::warn!("SMTP configuration: Configured with warnings");
        for warning in &smtp_status.warnings {
            logging::warn!("  - {}", warning);
        }
    } else {
        logging::warn!("SMTP configuration: Not configured (contact form will fail)");
        for error in &smtp_status.errors {
            logging::warn!("  - {}", error);
        }
    }

    // Determine the path to the Leptos configuration file (`Cargo.toml`).
    let config_path = std::env::var("LEPTOS_CONFIG_PATH").unwrap_or_else(|_| {
        let possible_paths = vec![
            "../Cargo.toml".to_string(),    // Common for `server` crate
            "Cargo.toml".to_string(),       // When running from root
            "./Cargo.toml".to_string(),     // Explicit current directory
            "../../Cargo.toml".to_string(), // When running from `target/debug`
        ];
        // Find the first existing Cargo.toml, otherwise default.
        possible_paths
            .into_iter()
            .find(|path| std::path::Path::new(path).exists())
            .unwrap_or_else(|| "../Cargo.toml".to_string())
    });

    // Load Leptos configuration.
    let Ok(conf) = get_configuration(Some(&config_path)) else {
        logging::error!("Failed to load configuration from: {}", config_path);
        return;
    };

    let mut leptos_options = conf.leptos_options;

    let leptos_site_addr_env = std::env::var("LEPTOS_SITE_ADDR").ok();
    let port_env = std::env::var("PORT").ok();

    let mut chosen_addr = choose_site_addr(
        leptos_options.site_addr,
        leptos_site_addr_env.as_deref(),
        port_env.as_deref(),
    );

    // Production deployments often probe the pod IP; binding to localhost will fail readiness.
    if !cfg!(debug_assertions) {
        if chosen_addr.ip().is_loopback() {
            chosen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), chosen_addr.port());
        }

        let has_leptos_site_addr = leptos_site_addr_env
            .as_deref()
            .map(str::trim)
            .is_some_and(|s| !s.is_empty());
        let has_port = port_env
            .as_deref()
            .map(str::trim)
            .is_some_and(|s| !s.is_empty());

        // If nothing is configured, default to DigitalOcean's expected port.
        if !has_leptos_site_addr && !has_port && chosen_addr.port() == 3007 {
            chosen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080);
        }
    }

    if chosen_addr != leptos_options.site_addr {
        logging::log!(
            "Overriding site addr {} -> {} (env)",
            leptos_options.site_addr,
            chosen_addr
        );
        leptos_options.site_addr = chosen_addr;
    }

    let leptos_options = Arc::new(leptos_options);
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(component); // Generate Leptos-specific routes.

    // Bind early so readiness probes can connect even while the database is initializing.
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(list) => list,
        Err(err) => {
            logging::error!("Failed to bind TCP listener to {addr}: {err}");
            return;
        }
    };

    let ready_router: Arc<OnceCell<Router>> = Arc::new(OnceCell::new());

    // Connect to SurrealDB and build the full router in the background.
    tokio::spawn({
        let ready_router = Arc::clone(&ready_router);
        let leptos_options = Arc::clone(&leptos_options);

        async move {
            loop {
                let db = match connect().await {
                    Ok(db) => db,
                    Err(err) => {
                        logging::error!("Failed to establish SurrealDB connection: {err:?}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                };

                let app_state = AppState {
                    db: Arc::new(db),
                    leptos_options: Arc::clone(&leptos_options),
                };

                // Initialize rate limiter: 100 requests per minute per IP.
                let rate_limiter = RateLimiter::new(100, 60);

                // Configure security headers based on environment.
                // Uses RUST_ENV to determine production vs development mode.
                let security_config = SecurityConfig::from_env();

                // Build the Axum router.
                let app: Router = Router::<AppState>::new()
                    // Integrate Leptos routes and server-side rendering.
                    .leptos_routes_with_context(
                        &app_state,
                        routes,
                        {
                            let app_state = app_state.clone();
                            move || provide_context(app_state.clone())
                        },
                        {
                            let leptos_options = Arc::clone(&leptos_options);
                            move || shell(Arc::clone(&leptos_options))
                        },
                    )
                    // Define additional API and utility routes.
                    // Backward-compatible redirects (older server-fn endpoints).
                    .route(
                        "/posts",
                        any(|| async { Redirect::temporary("/api/posts") }),
                    )
                    .route("/tags", any(|| async { Redirect::temporary("/api/tags") }))
                    .route("/post", any(|| async { Redirect::temporary("/api/post") }))
                    .route(
                        "/increment_views",
                        any(|| async { Redirect::temporary("/api/increment_views") }),
                    )
                    .route(
                        "/contact",
                        any(|| async { Redirect::temporary("/api/contact") }),
                    )
                    .route(
                        "/references",
                        any(|| async { Redirect::temporary("/api/references") }),
                    )
                    .route(
                        "/api/activities/create",
                        any(|| async { Redirect::temporary("/api/activities") }),
                    )
                    .route("/health", get(health_handler))
                    .route("/rss", get(rss_handler))
                    .route("/rss.xml", get(rss_handler))
                    .route("/sitemap.xml", get(sitemap_handler))
                    // Serve static assets.
                    .nest_service(
                        "/pkg", // WASM pkg assets
                        ServeDir::new(format!(
                            "{}/{}",
                            leptos_options.site_root.as_ref(),
                            leptos_options.site_pkg_dir.as_ref()
                        )),
                    )
                    .nest_service("/public", ServeDir::new(leptos_options.site_root.as_ref())) // General public assets
                    .nest_service(
                        "/fonts", // Web fonts
                        ServeDir::new(format!("{}/fonts", leptos_options.site_root.as_ref())),
                    )
                    // Apply Tower-HTTP middleware layers.
                    .layer(
                        tower::ServiceBuilder::new()
                            .layer(TraceLayer::new_for_http()) // Request tracing
                            .layer(axum::middleware::from_fn_with_state(
                                security_config,
                                security_headers_with_config,
                            )) // Apply environment-aware security HTTP headers
                            .layer(axum::middleware::from_fn(redirect_www)) // Enforce non-www redirect
                            .layer(axum::middleware::from_fn(move |req, next| {
                                // Per-IP rate limiting
                                let limiter = rate_limiter.clone();
                                async move { limiter.middleware(req, next).await }
                            })),
                    )
                    // Enable HTTP compression for responses larger than 1KB, excluding RSS feeds.
                    .layer(CompressionLayer::new().compress_when(
                        NotForContentType::new("application/rss+xml").and(SizeAbove::new(1024)),
                    ))
                    // Fallback handler for unmatched routes and error pages.
                    .fallback(leptos_axum::file_and_error_handler::<AppState, _>(
                        move |options| shell(Arc::new(options)),
                    ))
                    // Set the application state for Axum.
                    .with_state(app_state);

                if ready_router.set(app).is_ok() {
                    logging::log!("Application router initialized");
                }
                break;
            }
        }
    });

    let bootstrap_app = Router::new()
        .route("/health", get(health_handler))
        .fallback_service(tower::service_fn({
            let ready_router = Arc::clone(&ready_router);
            move |req: Request<Body>| {
                let ready_router = Arc::clone(&ready_router);
                async move {
                    if let Some(router) = ready_router.get() {
                        let res = router
                            .clone()
                            .into_service::<Body>()
                            .oneshot(req)
                            .await
                            // SAFETY: Router::oneshot returns Result<_, Infallible> - the error type
                            // `Infallible` can never be constructed, so this unwrap cannot panic
                            .unwrap();
                        Ok::<Response, Infallible>(res)
                    } else {
                        // SAFETY: Response::builder only fails with invalid header names/values;
                        // we use valid constants (SERVICE_UNAVAILABLE) so this cannot panic
                        Ok(Response::builder()
                            .status(StatusCode::SERVICE_UNAVAILABLE)
                            .body(Body::from("starting up"))
                            .unwrap())
                    }
                }
            }
        }));

    logging::log!("Listening on http://{}", &addr);

    let serve_result = axum::serve(listener, bootstrap_app.into_make_service()).await;
    match serve_result {
        Ok(_) => {
            logging::log!("Server shutdown gracefully");
        }
        Err(err) => {
            logging::error!("Failed to serve app: {}", err);
            logging::error!("Error details: {err:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::choose_site_addr;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn choose_site_addr_prefers_leptos_site_addr_env() {
        let config_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3007);
        let chosen = choose_site_addr(config_addr, Some("0.0.0.0:8080"), Some("9999"));
        assert_eq!(
            chosen,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080)
        );
    }

    #[test]
    fn choose_site_addr_falls_back_to_port_env() {
        let config_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3007);
        let chosen = choose_site_addr(config_addr, None, Some("8080"));
        assert_eq!(
            chosen,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8080)
        );
    }

    #[test]
    fn choose_site_addr_uses_config_addr_when_env_invalid() {
        let config_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3007);
        let chosen = choose_site_addr(config_addr, Some("not-an-addr"), Some("nope"));
        assert_eq!(chosen, config_addr);
    }
}
