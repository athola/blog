use crate::server::ports::Ports;
use std::net::SocketAddr;
use axum::{
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError,
};
use axum_extra::extract::Host;
use tracing::{debug, error, info, warn};

pub async fn redirect_http_to_https(ports: Ports) {
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
