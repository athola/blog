use axum::{
    body::Body,
    http::{Request, Response, StatusCode, header::{HeaderName, HeaderValue}},
    middleware::Next,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

/// Security headers middleware
/// Adds comprehensive security headers to all responses
pub async fn security_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // X-Frame-Options: Prevent clickjacking
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // X-Content-Type-Options: Prevent MIME type sniffing
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection: Enable XSS filter
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy: Control referrer information
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security (HSTS): Force HTTPS for 1 year
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content-Security-Policy: Restrict resource loading
    // Note: Adjusted for Leptos/WASM requirements
    let csp = [
        "default-src 'self'",
        "script-src 'self' 'wasm-unsafe-eval'",  // Required for WASM
        "style-src 'self' 'unsafe-inline'",       // Leptos inline styles
        "img-src 'self' data: https:",
        "font-src 'self' data:",
        "connect-src 'self'",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ].join("; ");

    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_str(&csp)
            .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
    );

    // Permissions-Policy: Disable unnecessary browser features
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=()"
        ),
    );

    Ok(response)
}

/// Rate limiting state
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of IP addresses to their request history
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// Maximum requests per window
    max_requests: usize,
    /// Time window in seconds
    window_secs: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed per window
    /// * `window_secs` - Time window in seconds
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    /// Check if a request from the given IP should be allowed
    async fn check_rate_limit(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        // Get or create request history for this IP
        let ip_requests = requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        ip_requests.retain(|&time| now.duration_since(time) < window);

        // Check if under the limit
        if ip_requests.len() < self.max_requests {
            ip_requests.push(now);
            true
        } else {
            false
        }
    }

    /// Middleware function for rate limiting
    pub async fn middleware(
        self,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response<Body>, StatusCode> {
        // Extract client IP address
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .unwrap_or("unknown")
            .trim()
            .to_string();

        // Check rate limit
        if !self.check_rate_limit(&ip).await {
            tracing::warn!("Rate limit exceeded for IP: {}", ip);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(next.run(req).await)
    }
}

/// Environment validation
/// Ensures all required environment variables are set for production
pub fn validate_production_env() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Required environment variables for production
    let required_vars = vec![
        "SURREAL_NS",
        "SURREAL_DB",
        "LEPTOS_SITE_ADDR",
    ];

    // Check for production mode
    let is_production = std::env::var("RUST_ENV")
        .unwrap_or_else(|_| "development".to_string())
        == "production";

    if is_production {
        for var in required_vars {
            if std::env::var(var).is_err() {
                errors.push(format!("Missing required environment variable: {}", var));
            }
        }

        // Check for database credentials
        let has_root_creds = std::env::var("SURREAL_ROOT_USER").is_ok()
            && std::env::var("SURREAL_ROOT_PASS").is_ok();
        let has_namespace_creds = std::env::var("SURREAL_NAMESPACE_USER").is_ok()
            && std::env::var("SURREAL_NAMESPACE_PASS").is_ok();
        let has_database_creds = std::env::var("SURREAL_USERNAME").is_ok()
            && std::env::var("SURREAL_PASSWORD").is_ok();

        if !has_root_creds && !has_namespace_creds && !has_database_creds {
            errors.push(
                "No database credentials found. Set one of: \
                SURREAL_ROOT_USER/PASS, SURREAL_NAMESPACE_USER/PASS, \
                or SURREAL_USERNAME/PASSWORD".to_string()
            );
        }

        // Validate password strength (minimum 8 characters for production)
        if let Ok(password) = std::env::var("SURREAL_PASSWORD") {
            if password.len() < 8 {
                errors.push(
                    "SURREAL_PASSWORD is too weak (minimum 8 characters required)".to_string()
                );
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(5, 60);

        // First 5 requests should be allowed
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("127.0.0.1").await);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, 60);

        // First 3 requests should be allowed
        for _ in 0..3 {
            assert!(limiter.check_rate_limit("192.168.1.1").await);
        }

        // 4th request should be blocked
        assert!(!limiter.check_rate_limit("192.168.1.1").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, 60);

        // Each IP should have its own limit
        assert!(limiter.check_rate_limit("10.0.0.1").await);
        assert!(limiter.check_rate_limit("10.0.0.2").await);
        assert!(limiter.check_rate_limit("10.0.0.1").await);
        assert!(limiter.check_rate_limit("10.0.0.2").await);

        // Now both should be at limit
        assert!(!limiter.check_rate_limit("10.0.0.1").await);
        assert!(!limiter.check_rate_limit("10.0.0.2").await);
    }
}
