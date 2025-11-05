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

// Health check handler
async fn health_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "blog-api",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

#[tokio::main]
async fn main() {
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

    if dotenv().is_err() {
        logging::warn!("There is no corresponding .env file");
    }

    // Validate production environment
    if let Err(errors) = validate_production_env() {
        for error in errors {
            logging::error!("Environment validation error: {}", error);
        }
        logging::warn!("Continuing despite environment validation errors (development mode)");
    }

    // Determine the configuration file path
    let config_path = std::env::var("LEPTOS_CONFIG_PATH").unwrap_or_else(|_| {
        // Try multiple possible locations for Cargo.toml
        let possible_paths = vec![
            "../Cargo.toml".to_string(),    // When running from server directory
            "Cargo.toml".to_string(),       // When running from root directory
            "./Cargo.toml".to_string(),     // Explicit current directory
            "../../Cargo.toml".to_string(), // When running from target/debug
        ];

        possible_paths
            .into_iter()
            .find(|path| std::path::Path::new(path).exists())
            .unwrap_or_else(|| "../Cargo.toml".to_string()) // Default to original
    });

    let Ok(conf) = get_configuration(Some(&config_path)) else {
        logging::error!("Failed to get configuration from: {}", config_path);
        return;
    };

    let leptos_options = Arc::new(conf.leptos_options);
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(component);

    let db = match connect().await {
        Ok(db) => db,
        Err(err) => {
            logging::error!("Failed to establish SurrealDB connection: {:?}", err);
            return;
        }
    };
    let app_state = AppState {
        db: Arc::new(db),
        leptos_options: Arc::clone(&leptos_options),
    };

    // Initialize rate limiter: 100 requests per minute per IP
    let rate_limiter = RateLimiter::new(100, 60);

    let app =
        Router::<AppState>::new()
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
            .route("/health", get(health_handler))
            .route("/rss", get(rss_handler))
            .route("/rss.xml", get(rss_handler))
            .route("/sitemap.xml", get(sitemap_handler))
            .nest_service(
                "/pkg",
                ServeDir::new(format!(
                    "{}/{}",
                    leptos_options.site_root.as_ref(),
                    leptos_options.site_pkg_dir.as_ref()
                )),
            )
            .nest_service("/public", ServeDir::new(leptos_options.site_root.as_ref()))
            .nest_service(
                "/fonts",
                ServeDir::new(format!("{}/fonts", leptos_options.site_root.as_ref())),
            )
            .layer(
                tower::ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(axum::middleware::from_fn(security_headers))
                    .layer(axum::middleware::from_fn(redirect_www))
                    .layer(axum::middleware::from_fn(move |req, next| {
                        let limiter = rate_limiter.clone();
                        async move { limiter.middleware(req, next).await }
                    })),
            )
            .layer(CompressionLayer::new().compress_when(
                NotForContentType::new("application/rss+xml").and(SizeAbove::new(1024)),
            ))
            .fallback(leptos_axum::file_and_error_handler::<AppState, _>(
                move |options| shell(Arc::new(options)),
            ))
            .with_state(app_state);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(list) => list,
        Err(err) => {
            logging::error!("Failed to bind tcp listener to {}: {}", &addr, err);
            return;
        }
    };
    logging::log!("Listening on http://{}", &addr);

    // Add more detailed error handling for the serve function
    let serve_result = axum::serve(listener, app.into_make_service()).await;
    match serve_result {
        Ok(_) => {
            logging::log!("Server shutdown gracefully");
        }
        Err(err) => {
            logging::error!("Failed to serve app: {}", err);
            logging::error!("Error details: {:?}", err);
        }
    }
}
