use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
};

pub async fn redirect_www(req: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode> {
    if let Some(host) = req.headers().get("host")
        && let Ok(host) = host.to_str()
        && host.starts_with("www.") {
            let new_host = host.trim_start_matches("www.");
            if let Some(path_query) = req.uri().path_and_query() {
                let new_uri = format!("https://{}{}", new_host, path_query.as_str(),);
                let response = Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header("location", new_uri)
                    .body(Body::empty())
                    .unwrap();
                return Ok(response);
            }
        }
    Ok(next.run(req).await)
}
