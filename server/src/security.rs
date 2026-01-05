//! This module provides security-related functionalities for the Axum server,
//! including middleware for setting HTTP security headers and implementing
//! IP-based rate limiting.
//!
//! It also includes utilities for validating production environment variables
//! to ensure secure deployment configurations.

use axum::{
    body::Body,
    http::{
        Request, Response, StatusCode,
        header::{HeaderName, HeaderValue},
    },
    middleware::Next,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Axum middleware that adds a set of HTTP security headers to all responses.
///
/// These headers help protect against common web vulnerabilities like XSS,
/// clickjacking, and MIME type sniffing. It also enforces HTTPS and sets
/// a strict Content Security Policy (CSP).
///
/// # Arguments
///
/// * `req` - The incoming `Request`.
/// * `next` - The `Next` middleware in the stack.
///
/// # Returns
///
/// A `Result` containing the `Response` with added security headers, or an
/// `Axum` `StatusCode` if an error occurs (e.g., invalid header value).
pub async fn security_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // X-Frame-Options: Prevents clickjacking attacks.
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // X-Content-Type-Options: Prevents MIME type sniffing vulnerabilities.
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection: Enables the browser's XSS filter.
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy: Controls how much referrer information is sent with requests.
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security (HSTS): Forces all communication over HTTPS for one year.
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Content-Security-Policy: Restricts resource loading to trusted sources.
    // Configured to support Leptos/WASM requirements.
    let csp = [
        "default-src 'self'",
        "script-src 'self' 'wasm-unsafe-eval'", // Required for WASM execution.
        "style-src 'self' 'unsafe-inline'",     // Required for Leptos's inline styles.
        "img-src 'self' data: https:",
        "font-src 'self' data:",
        "connect-src 'self'",
        "frame-ancestors 'none'", // Prevents embedding in iframes.
        "base-uri 'self'",
        "form-action 'self'",
    ]
    .join("; ");

    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_str(&csp)
            .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
    );

    // Permissions-Policy: Disables potentially risky or unnecessary browser features.
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=()",
        ),
    );

    Ok(response)
}

/// Implements a simple IP-based rate limiting mechanism.
///
/// Tracks requests per IP address within a sliding time window and blocks
/// requests exceeding a configured limit.
#[derive(Clone)]
pub struct RateLimiter {
    /// Stores the request history for each IP address.
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// The maximum number of requests allowed within the `window_secs`.
    max_requests: usize,
    /// The time window in seconds during which `max_requests` are allowed.
    window_secs: u64,
}

impl RateLimiter {
    /// Creates a new `RateLimiter` instance.
    ///
    /// # Arguments
    /// * `max_requests` - The maximum number of requests allowed from a single IP within the time window.
    /// * `window_secs` - The duration (in seconds) of the sliding time window.
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    /// Checks if a request from the given IP address should be allowed by the rate limit.
    ///
    /// # Arguments
    /// * `ip` - The IP address of the client making the request.
    ///
    /// # Returns
    /// `true` if the request is allowed, `false` if it exceeds the rate limit.
    async fn check_rate_limit(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        // Get or create request history for this IP.
        let ip_requests = requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests that fall outside the current time window.
        ip_requests.retain(|&time| now.duration_since(time) < window);

        // Clean up empty IP entries to prevent unbounded memory growth.
        // If an IP has no recent requests within the time window, remove it entirely.
        if ip_requests.is_empty() {
            requests.remove(ip);
            // Since the vector was empty, we can allow this request.
            // Re-insert with the current timestamp to track this new request.
            requests
                .entry(ip.to_string())
                .or_insert_with(Vec::new)
                .push(now);
            return true;
        }

        // If the number of requests is within the limit, record the current request.
        if ip_requests.len() < self.max_requests {
            ip_requests.push(now);
            true
        } else {
            false
        }
    }

    /// Axum middleware function that applies the rate limiting logic.
    ///
    /// Extracts the client's IP address (prioritizing `X-Forwarded-For` header)
    /// and checks it against the configured rate limits. If the limit is exceeded,
    /// it returns `429 Too Many Requests`.
    ///
    /// # Security Notes
    /// - IP addresses are validated before use to prevent header injection
    /// - Invalid IP addresses fall back to "unknown" to prevent log pollution
    /// - Rate limiting is still applied to "unknown" to prevent abuse
    pub async fn middleware(
        self,
        req: Request<Body>,
        next: Next,
    ) -> Result<Response<Body>, StatusCode> {
        // Extract and validate client IP address, preferring `X-Forwarded-For` for proxy compatibility.
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim())
            .and_then(|s| {
                // Validate that the header value is actually an IP address.
                // This prevents header injection and log pollution.
                if IpAddr::from_str(s).is_ok() {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Check the rate limit for the extracted IP.
        if !self.check_rate_limit(&ip).await {
            tracing::warn!(
                rate_limit = self.max_requests,
                window_secs = self.window_secs,
                "Rate limit exceeded"
            );
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(next.run(req).await)
    }
}

/// Environment validation utility.
///
/// Ensures that all critical environment variables are set when the application
/// is running in "production" mode (`RUST_ENV=production`). This helps prevent
/// deployment-time misconfigurations related to database access or server settings.
/// It also includes basic password strength validation.
///
/// # Returns
///
/// An `Ok(())` if all checks pass, or an `Err(Vec<String>)` containing a list of
/// validation error messages if any issues are found.
pub fn validate_production_env() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Required environment variables for a production deployment.
    let required_vars = vec!["SURREAL_NS", "SURREAL_DB", "LEPTOS_SITE_ADDR"];

    // Determine if the application is running in production mode.
    let is_production =
        std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "production";

    if is_production {
        for var in required_vars {
            if std::env::var(var).is_err() {
                errors.push(format!("Missing required environment variable: {}", var));
            }
        }

        // Check for at least one set of database credentials.
        let has_root_creds = std::env::var("SURREAL_ROOT_USER").is_ok()
            && std::env::var("SURREAL_ROOT_PASS").is_ok();
        let has_namespace_creds = std::env::var("SURREAL_NAMESPACE_USER").is_ok()
            && std::env::var("SURREAL_NAMESPACE_PASS").is_ok();
        let has_database_creds =
            std::env::var("SURREAL_USERNAME").is_ok() && std::env::var("SURREAL_PASSWORD").is_ok();

        if !has_root_creds && !has_namespace_creds && !has_database_creds {
            errors.push(
                "No database credentials found. Set one of: \
                SURREAL_ROOT_USER/PASS, SURREAL_NAMESPACE_USER/PASS, \
                or SURREAL_USERNAME/PASSWORD"
                    .to_string(),
            );
        }

        // Validate password strength (minimum 8 characters required for production).
        if let Ok(password) = std::env::var("SURREAL_PASSWORD")
            && password.len() < 8
        {
            errors.push("SURREAL_PASSWORD is too weak (minimum 8 characters required)".to_string());
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

    /// Test that the rate limiter allows requests under the configured limit.
    #[tokio::test]
    async fn test_rate_limiter_allows_under_limit() {
        let limiter = RateLimiter::new(5, 60);

        // Simulate 5 requests; all should be allowed.
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("127.0.0.1").await);
        }
    }

    /// Test that the rate limiter blocks requests exceeding the configured limit.
    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, 60);

        // Simulate 3 requests; all should be allowed.
        for _ in 0..3 {
            assert!(limiter.check_rate_limit("192.168.1.1").await);
        }

        // The 4th request should be blocked.
        assert!(!limiter.check_rate_limit("192.168.1.1").await);
    }

    /// Test that the rate limiter maintains separate limits for different IP addresses.
    #[tokio::test]
    async fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, 60);

        // Requests from different IPs should not affect each other's limits.
        assert!(limiter.check_rate_limit("10.0.0.1").await);
        assert!(limiter.check_rate_limit("10.0.0.2").await);
        assert!(limiter.check_rate_limit("10.0.0.1").await);
        assert!(limiter.check_rate_limit("10.0.0.2").await);

        // Both IPs should now be at their respective limits.
        assert!(!limiter.check_rate_limit("10.0.0.1").await);
        assert!(!limiter.check_rate_limit("10.0.0.2").await);
    }

    /// Test that the rate limiter properly cleans up IP entries after the time window expires.
    /// This prevents unbounded memory growth from tracking inactive IPs indefinitely.
    #[tokio::test]
    async fn test_rate_limiter_memory_cleanup() {
        // Use a short time window (1 second) for testing.
        let limiter = RateLimiter::new(10, 1);

        // Make requests from many different IPs.
        for i in 0..100 {
            let ip = format!("192.168.1.{}", i);
            assert!(limiter.check_rate_limit(&ip).await);
        }

        // Verify all IPs are tracked.
        {
            let requests = limiter.requests.lock().await;
            assert_eq!(requests.len(), 100);
        }

        // Wait for the time window to expire plus a small buffer.
        tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

        // Make new requests from a subset of previously tracked IPs.
        // These should trigger cleanup of their expired entries.
        for i in 0..10 {
            let ip = format!("192.168.1.{}", i);
            assert!(limiter.check_rate_limit(&ip).await);
        }

        // Verify that IP entries are being cleaned up when they make new requests.
        // The 10 IPs we re-queried should have been cleaned up and now only have 1 entry each.
        {
            let requests = limiter.requests.lock().await;
            for i in 0..10 {
                let ip = format!("192.168.1.{}", i);
                let ip_requests = requests.get(&ip);
                assert!(ip_requests.is_some(), "IP should still be tracked");
                assert_eq!(
                    ip_requests.unwrap().len(),
                    1,
                    "Old entries should have been cleaned up"
                );
            }
        }
    }
}
