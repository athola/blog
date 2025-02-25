#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
#![allow(clippy::multiple_crate_versions)]
cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use std::{net::SocketAddr, path::PathBuf};
        use axum::{
            http::StatusCode,
            routing::get,
            Router,
        };
        use axum_server::tls_rustls::RustlsConfig;
        use tracing::{debug, error, info, warn};
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tls_rustls=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ports = server::ports::Ports {
        http: 8080,
        https: 4443,
    };
    // optional: spawn a second server to redirect http requests to this server
    tokio::spawn(server::redirect::redirect_http_to_https(ports));

    // configure certificate and private keys used by https
    let config_result = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("blog.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("self_signed_certs")
            .join("blog.key"),
    )
    .await;
    let config = match config_result {
        Ok(v) => {
            info!("Successfully read from .pem file {v:?}");
            v
        }
        Err(e) => {
            error!("Error reading from .pem file: {e:?}");
            return;
        }
    };

    let app = Router::new().route("/", get(route_handler));

    // run https server
    let addr = SocketAddr::from(([127, 0, 0, 1], ports.https));
    debug!("listening on {}", addr);
    match axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
    {
        Ok(v) => info!("Bound to axum server {v:?}"),
        Err(e) => error!("Error binding to axum server: {e:?}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::unused_async)]
async fn route_handler() -> Result<String, StatusCode> {
    Ok("Hello, World!".to_owned())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Ignore for now
}
