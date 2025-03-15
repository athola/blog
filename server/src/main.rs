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

    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            move || provide_context(app_state.clone()),
            move || shell(leptos_options.clone()),
        )
        .route("/rss.xml", get(rss_handler))
        .route("/sitemap.xml", get(sitemap_handler))
        .layer(
            tower::ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(redirect_www)),
        )
        .layer(
            CompressionLayer::new()
                .quality(CompressionLevel::Default)
                .compress_when(
                    SizeAbove::new(1500)
                        .and(NotForContentType::GRPC)
                        .and(NotForContentType::IMAGES)
                        .and(NotForContentType::const_new("application/xml"))
                        .and(NotForContentType::const_new("application/javascript"))
                        .and(NotForContentType::const_new("application/wasm"))
                        .and(NotForContentType::const_new("test/css"))
                ),
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(list) => list,
        Err(_err) => {
            logging::error!("Failed to bind tcp listener to {}: {}", &addr, _err);
            return;
        }
    };
    logging::log!("Listening on http://{}", &addr);
    if let Err(_err) = axum::serve(listener, app.into_make_service()).await {
        logging::error!("Failed to serve app: {}", _err);
    };
}
