//! This module is the main entry point for the Axum server, serving the Leptos
//! application.
//!
//! It handles server initialization, configuration loading, database connections,
//! routing, middleware setup (security headers, rate limiting, compression),
//! and static asset serving.

mod redirect;
mod security;
mod utils;

use app::{component, shell, types::AppState};
use axum::{Router, http::StatusCode, response::Json, routing::get};
use dotenvy::dotenv;
use leptos::logging;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes as _, generate_route_list};
use leptos_config::get_configuration;
use redirect::redirect_www;
use security::{RateLimiter, security_headers, validate_production_env};
use serde_json::json;

use std::sync::Arc;
use tower_http::compression::predicate::{NotForContentType, SizeAbove};
use tower_http::compression::{CompressionLayer, Predicate as _};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utils::{connect, rss_handler, sitemap_handler};

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
    if dotenv().is_err() {
        logging::warn!(
            "No .env file found. Proceeding without loading environment variables from file."
        );
    }

    // Validate essential environment variables for production.
    if let Err(errors) = validate_production_env() {
        for error in errors {
            logging::error!("Environment validation error: {}", error);
        }
        logging::warn!(
            "Continuing despite environment validation errors, assuming development mode."
        );
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

    let leptos_options = Arc::new(conf.leptos_options);
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(component); // Generate Leptos-specific routes.

    // Establish connection to SurrealDB.
    let db = match connect().await {
        Ok(db) => db,
        Err(err) => {
            logging::error!("Failed to establish SurrealDB connection: {err:?}");
            return;
        }
    };
    // Bundle application state for Axum.
    let app_state = AppState {
        db: Arc::new(db),
        leptos_options: Arc::clone(&leptos_options),
    };

    // Initialize rate limiter: 100 requests per minute per IP.
    let rate_limiter = RateLimiter::new(100, 60);

    // Build the Axum router.
    let app =
        Router::<AppState>::new()
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
                    .layer(axum::middleware::from_fn(security_headers)) // Apply security HTTP headers
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

    // Bind the TCP listener and start serving the application.
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(list) => list,
        Err(err) => {
            logging::error!("Failed to bind TCP listener to {addr}: {err}");
            return;
        }
    };
    logging::log!("Listening on http://{}", &addr);

    let serve_result = axum::serve(listener, app.into_make_service()).await;
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
