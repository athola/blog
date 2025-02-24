#![warn(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]
use std::{net::SocketAddr, path::PathBuf};
use axum::{
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::get,
    BoxError, Router,
};
use axum_extra::extract::Host;
use axum_server::tls_rustls::RustlsConfig;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tls_rustls=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let ports = Ports {
        http: 8080,
        https: 4443,
    };
    // optional: spawn a second server to redirect http requests to this server
    tokio::spawn(redirect_http_to_https(ports));

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

    let app = Router::new().route("/", get(handler));

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

#[allow(clippy::unused_async)]
async fn handler() -> Result<String, StatusCode> {
    Ok("Hello, World!".to_owned())
}

async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: &str, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut uri_parts = uri.into_parts();

        uri_parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if uri_parts.path_and_query.is_none() {
            uri_parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        uri_parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(uri_parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(&host, uri, ports) {
            Ok(redir_uri) => Ok(Redirect::permanent(&redir_uri.to_string())),
            Err(error) => {
                warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    debug!("http redirect listening on {}", addr);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(tcp_list) => tcp_list,
        Err(e) => {
            error!("Failed to bind {addr:?} to tcp listener: {e:?}");
            return
        },
    };

    match axum::serve(listener, redirect.into_make_service()).await
    {
        Ok(v) => info!("Serving {addr:?} on axum server {v:?}"),
        Err(e) => error!("Failed to serve {addr:?} on axum server: {e:?}"),
    }
}
