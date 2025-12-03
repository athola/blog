//! This module provides an Axum middleware for handling "www" to non-"www"
//! URL redirects.
//!
//! It ensures that requests to `www.yourdomain.com` are permanently redirected
//! to `yourdomain.com`, improving SEO and providing a consistent user experience.

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};

/// An Axum middleware that redirects "www" subdomains to their non-"www" counterparts.
///
/// If a request's `Host` header starts with "www.", this middleware constructs a
/// new URI with the "www." prefix removed and issues a `301 Moved Permanently`
/// redirect. Otherwise, it passes the request to the next middleware in the stack.
///
/// # Arguments
///
/// * `req` - The incoming `Request`.
/// * `next` - The `Next` middleware in the stack.
///
/// # Returns
///
/// A `Result` containing either the `Response` (a redirect or the next handler's response)
/// or an `Axum` `StatusCode` if an unrecoverable error occurs during processing.
pub async fn redirect_www(req: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode> {
    // Extract the host header and check if it starts with "www.".
    if let Some(host) = req.headers().get("host")
        && let Ok(host) = host.to_str()
        && host.starts_with("www.")
    {
        let new_host = host.trim_start_matches("www.");
        // If a path and query exist, construct the new URI for redirection.
        if let Some(path_query) = req.uri().path_and_query() {
            let new_uri = format!("https://{}{}", new_host, path_query.as_str(),);
            // Create a 301 Moved Permanently response.
            let response = Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header("location", new_uri)
                .body(Body::empty())
                .unwrap(); // `unwrap()` is safe here as `Body::empty()` is valid.
            return Ok(response);
        }
    }
    // If no "www." prefix or no path/query, proceed to the next middleware.
    Ok(next.run(req).await)
}
