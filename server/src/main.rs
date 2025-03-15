mod redirect;
mod utils;

use app::{component, shell, types::AppState};
use axum::{routing::get, Router};
use dotenvy::dotenv;
use leptos::logging;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use redirect::redirect_www;
use tower_http::compression::predicate::{NotForContentType, SizeAbove};
use tower_http::compression::{CompressionLayer, Predicate};
use tower_http::trace::TraceLayer;
use tower_http::CompressionLevel;
use utils::{connect, rss_handler, sitemap_handler};


#[tokio::main]
async fn main() {
    let tracing_level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt)
        .with_file(true)
        .with_line_number(true)
        .with_max_level(tracing_level)
        .init();

    let env_result = dotenv();
    if env_result.is_err() {
        logging::warn!("There is no corresponding .env file");
    }

    let conf = match get_configuration(None) {
        Ok(cfg) => cfg,
        Err(_) => {
            logging.error!("Failed to get configuration");
            return
        }
    };

    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(component);

    let db = connect().await;
    let app_state = AppState {
        db,
        leptos_options: leptos_options.clone(),
    };
}
