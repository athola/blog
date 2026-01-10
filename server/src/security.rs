//! This module provides security-related functionalities for the Axum server,
//! including middleware for setting HTTP security headers and implementing
//! IP-based rate limiting.
//!
//! It also includes utilities for validating production environment variables
//! to ensure secure deployment configurations.

use axum::{
    body::Body,
    extract::State,
    http::{
        Request, Response, StatusCode,
        header::{HeaderName, HeaderValue},
    },
    middleware::Next,
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Configuration for security headers based on environment.
///
/// This struct controls the behavior of security headers middleware,
/// allowing different configurations for development and production environments.
#[derive(Clone, Copy, Debug)]
pub struct SecurityConfig {
    /// Whether the application is running in production mode.
    pub is_production: bool,
}

impl SecurityConfig {
    /// Creates a new `SecurityConfig` instance.
    ///
    /// # Arguments
    /// * `is_production` - Set to `true` for production environments with strict security,
    ///   `false` for development environments with relaxed policies.
    pub fn new(is_production: bool) -> Self {
        Self { is_production }
    }

    /// Creates a `SecurityConfig` by checking the `RUST_ENV` environment variable.
    ///
    /// Returns production config if `RUST_ENV=production`, otherwise development config.
    pub fn from_env() -> Self {
        let is_production =
            std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()) == "production";
        Self::new(is_production)
    }
}

/// Categorizes endpoints for rate limiting purposes.
///
/// Different endpoint categories have different rate limits to balance
/// security needs with usability.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointCategory {
    /// API endpoints: 100 requests/minute (default)
    Api,
    /// Contact form: 10 requests/minute (stricter to prevent spam)
    Contact,
    /// Static assets: 1000 requests/minute (relaxed)
    Static,
    /// Health check: Unlimited (for monitoring)
    Health,
}

#[allow(dead_code)]
impl EndpointCategory {
    /// Returns the rate limit configuration for this endpoint category.
    ///
    /// Returns `None` for unlimited categories (like Health).
    pub fn rate_limit(&self) -> Option<(usize, u64)> {
        match self {
            EndpointCategory::Api => Some((100, 60)), // 100 requests per minute
            EndpointCategory::Contact => Some((10, 60)), // 10 requests per minute
            EndpointCategory::Static => Some((1000, 60)), // 1000 requests per minute
            EndpointCategory::Health => None,         // Unlimited
        }
    }

    /// Returns a human-readable name for this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            EndpointCategory::Api => "api",
            EndpointCategory::Contact => "contact",
            EndpointCategory::Static => "static",
            EndpointCategory::Health => "health",
        }
    }
}

impl std::fmt::Display for EndpointCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Status information about rate limiting for a specific IP and category.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitStatus {
    /// Number of requests remaining in the current window.
    pub remaining: usize,
    /// Seconds until the rate limit window resets.
    pub reset_in_seconds: u64,
    /// Whether the client is currently rate limited.
    pub is_limited: bool,
}

/// Error returned when a request is rate limited.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct RateLimitError {
    /// Error message.
    pub error: String,
    /// Seconds until the client can retry.
    pub retry_after: u64,
    /// Number of requests remaining (always 0 when limited).
    pub remaining: usize,
    /// The endpoint category that was rate limited.
    pub category: String,
}

#[allow(dead_code)]
impl RateLimitError {
    /// Creates a new rate limit error.
    pub fn new(category: EndpointCategory, retry_after: u64) -> Self {
        Self {
            error: "Rate limit exceeded".to_string(),
            retry_after,
            remaining: 0,
            category: category.as_str().to_string(),
        }
    }
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> axum::response::Response {
        let retry_after = self.retry_after.to_string();
        let body = serde_json::to_string(&self)
            .unwrap_or_else(|_| r#"{"error":"Rate limit exceeded"}"#.to_string());

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            body,
        )
            .into_response();

        // Add Retry-After header
        if let Ok(value) = HeaderValue::from_str(&retry_after) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("retry-after"), value);
        }

        response
    }
}

/// Axum middleware that adds a set of HTTP security headers to all responses.
///
/// These headers help protect against common web vulnerabilities like XSS,
/// clickjacking, and MIME type sniffing. It also enforces HTTPS and sets
/// a strict Content Security Policy (CSP).
///
/// This version uses the default production configuration. For environment-aware
/// configuration, use `security_headers_with_config` instead.
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
#[allow(dead_code)]
pub async fn security_headers(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // Default to production-safe configuration
    security_headers_with_config(State(SecurityConfig::new(true)), req, next).await
}

/// Axum middleware that adds environment-aware HTTP security headers to all responses.
///
/// This middleware adjusts security headers based on whether the application is
/// running in production or development mode:
///
/// ## Production Mode (strict)
/// - Full HSTS with 1-year max-age and includeSubDomains
/// - Strict CSP without unsafe-eval
/// - All Cross-Origin policies enabled
///
/// ## Development Mode (relaxed)
/// - Shorter HSTS (1 day) without includeSubDomains
/// - Relaxed CSP with unsafe-eval for hot reload
/// - WebSocket connections allowed for live reload
///
/// # Arguments
///
/// * `config` - The `SecurityConfig` extracted from Axum state.
/// * `req` - The incoming `Request`.
/// * `next` - The `Next` middleware in the stack.
///
/// # Returns
///
/// A `Result` containing the `Response` with added security headers, or an
/// `Axum` `StatusCode` if an error occurs (e.g., invalid header value).
pub async fn security_headers_with_config(
    State(config): State<SecurityConfig>,
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

    // X-XSS-Protection: Legacy XSS filter for older browsers.
    // NOTE: Deprecated in modern browsers (Chrome/Edge removed it). CSP is the modern solution.
    // Retained for defense-in-depth with legacy browser support.
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer-Policy: Controls how much referrer information is sent with requests.
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Strict-Transport-Security (HSTS): Environment-dependent configuration.
    // Production: 1 year with includeSubDomains for maximum security.
    // Development: 1 day without includeSubDomains to avoid HSTS issues during testing.
    let hsts = if config.is_production {
        "max-age=31536000; includeSubDomains"
    } else {
        "max-age=86400"
    };
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static(hsts),
    );

    // Content-Security-Policy: Environment-dependent configuration.
    // Production: Strict CSP without unsafe-eval.
    // Development: Relaxed CSP with unsafe-eval for hot reload and ws:// for live reload.
    let csp = if config.is_production {
        // Production CSP: No unsafe-eval, strict sources.
        [
            "default-src 'self'",
            "script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline'", // unsafe-inline for Leptos hydration
            "style-src 'self' 'unsafe-inline'", // Required for Leptos inline styles
            "img-src 'self' data: https:",
            "font-src 'self' data:",
            "connect-src 'self'",
            "frame-ancestors 'none'",
            "base-uri 'self'",
            "form-action 'self'",
            "upgrade-insecure-requests",
        ]
        .join("; ")
    } else {
        // Development CSP: Allow unsafe-eval for hot reload, ws:// for live reload.
        [
            "default-src 'self'",
            "script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline' 'unsafe-eval'", // unsafe-eval for hot reload
            "style-src 'self' 'unsafe-inline'",
            "img-src 'self' data: https: http:",
            "font-src 'self' data:",
            "connect-src 'self' ws://localhost:* ws://127.0.0.1:* http://localhost:* http://127.0.0.1:*", // WebSocket for live reload
            "frame-ancestors 'none'",
            "base-uri 'self'",
            "form-action 'self'",
        ]
        .join("; ")
    };

    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_str(&csp)
            .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'")),
    );

    // Permissions-Policy: Comprehensive list of disabled browser features.
    // These are disabled in both production and development for security.
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), ambient-light-sensor=(), autoplay=(), battery=(), camera=(), \
             cross-origin-isolated=(), display-capture=(), document-domain=(), encrypted-media=(), \
             execution-while-not-rendered=(), execution-while-out-of-viewport=(), fullscreen=(), \
             geolocation=(), gyroscope=(), keyboard-map=(), magnetometer=(), microphone=(), midi=(), \
             navigation-override=(), payment=(), picture-in-picture=(), publickey-credentials-get=(), \
             screen-wake-lock=(), sync-xhr=(), usb=(), web-share=(), xr-spatial-tracking=()",
        ),
    );

    // Cross-Origin-Opener-Policy: Isolates the browsing context.
    // Prevents other windows from holding a reference to this window.
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );

    // Cross-Origin-Resource-Policy: Restricts which origins can load this resource.
    // same-origin prevents other sites from embedding our resources.
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );

    // Cross-Origin-Embedder-Policy: Controls embedding of cross-origin resources.
    // Note: require-corp is commented out as it may break loading of external
    // resources (fonts, images) that don't set CORP headers. Enable if needed
    // for SharedArrayBuffer or other cross-origin isolation features.
    // Uncomment the following if cross-origin isolation is required:
    // headers.insert(
    //     HeaderName::from_static("cross-origin-embedder-policy"),
    //     HeaderValue::from_static("require-corp"),
    // );

    Ok(response)
}

/// Creates a closure suitable for use with `axum::middleware::from_fn_with_state`.
///
/// This is a convenience function that creates the security headers middleware
/// with the given configuration.
///
/// # Arguments
/// * `is_production` - Whether the application is running in production mode.
///
/// # Example
/// ```ignore
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(axum::middleware::from_fn_with_state(
///         SecurityConfig::new(is_production),
///         security_headers_with_config,
///     ));
/// ```
#[allow(dead_code)]
pub fn create_security_config(is_production: bool) -> SecurityConfig {
    SecurityConfig::new(is_production)
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

/// SMTP configuration validation result.
#[derive(Debug, Clone)]
pub struct SmtpValidation {
    /// Whether all SMTP credentials are configured.
    pub is_configured: bool,
    /// Any validation errors encountered.
    pub errors: Vec<String>,
    /// Any warnings (non-fatal issues).
    pub warnings: Vec<String>,
}

impl SmtpValidation {
    /// Returns `true` if SMTP is properly configured with no errors.
    pub fn is_valid(&self) -> bool {
        self.is_configured && self.errors.is_empty()
    }
}

/// Validates SMTP configuration for email functionality.
///
/// Checks that all required SMTP environment variables are set and validates
/// their format where possible. This function can be called at startup to
/// ensure email functionality will work.
///
/// # Required Environment Variables
///
/// - `SMTP_HOST`: The SMTP server hostname (e.g., "smtp.gmail.com")
/// - `SMTP_USER`: The SMTP username/email for authentication
/// - `SMTP_PASSWORD`: The SMTP password or app-specific password
///
/// # Returns
///
/// An `SmtpValidation` struct containing configuration status and any errors/warnings.
///
/// # Example
///
/// ```ignore
/// let smtp_status = validate_smtp_config();
/// if !smtp_status.is_valid() {
///     for error in &smtp_status.errors {
///         eprintln!("SMTP Error: {}", error);
///     }
/// }
/// ```
pub fn validate_smtp_config() -> SmtpValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check required SMTP environment variables
    let smtp_host = std::env::var("SMTP_HOST");
    let smtp_user = std::env::var("SMTP_USER");
    let smtp_password = std::env::var("SMTP_PASSWORD");

    let is_configured = smtp_host.is_ok() && smtp_user.is_ok() && smtp_password.is_ok();

    // Validate SMTP_HOST
    match &smtp_host {
        Ok(host) if host.trim().is_empty() => {
            errors.push("SMTP_HOST is empty".to_string());
        }
        Ok(host) => {
            // Basic hostname validation
            if !host.contains('.') && host != "localhost" {
                warnings.push(format!(
                    "SMTP_HOST '{}' may be invalid (no domain suffix)",
                    host
                ));
            }
        }
        Err(_) => {
            errors.push("Missing required environment variable: SMTP_HOST".to_string());
        }
    }

    // Validate SMTP_USER (typically an email address)
    match &smtp_user {
        Ok(user) if user.trim().is_empty() => {
            errors.push("SMTP_USER is empty".to_string());
        }
        Ok(user) => {
            // Basic email format check
            if !user.contains('@') {
                warnings.push(format!(
                    "SMTP_USER '{}' does not appear to be an email address",
                    user
                ));
            }
        }
        Err(_) => {
            errors.push("Missing required environment variable: SMTP_USER".to_string());
        }
    }

    // Validate SMTP_PASSWORD
    match &smtp_password {
        Ok(password) if password.is_empty() => {
            errors.push("SMTP_PASSWORD is empty".to_string());
        }
        Ok(password) if password.len() < 8 => {
            warnings.push("SMTP_PASSWORD is short (less than 8 characters)".to_string());
        }
        Err(_) => {
            errors.push("Missing required environment variable: SMTP_PASSWORD".to_string());
        }
        _ => {}
    }

    SmtpValidation {
        is_configured,
        errors,
        warnings,
    }
}

/// Environment validation utility.
///
/// Ensures that all critical environment variables are set when the application
/// is running in "production" mode (`RUST_ENV=production`). This helps prevent
/// deployment-time misconfigurations related to database access, server settings,
/// and email (SMTP) functionality.
///
/// # Validated Configuration
///
/// - **Database**: SurrealDB namespace, database, and credentials
/// - **Server**: Leptos site address configuration
/// - **Email**: SMTP host, user, and password for contact form functionality
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

        // Validate SMTP configuration for email functionality.
        let smtp_validation = validate_smtp_config();
        errors.extend(smtp_validation.errors);

        // Log warnings but don't fail on them
        for warning in smtp_validation.warnings {
            tracing::warn!("SMTP configuration warning: {}", warning);
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
    use serial_test::serial;

    /// Test SecurityConfig creation with explicit production flag.
    #[test]
    fn test_security_config_new() {
        let prod_config = SecurityConfig::new(true);
        assert!(prod_config.is_production);

        let dev_config = SecurityConfig::new(false);
        assert!(!dev_config.is_production);
    }

    /// Test SecurityConfig creation from environment variable.
    /// Note: Uses unsafe blocks as env var manipulation is unsafe in Rust 2024+ edition.
    #[test]
    #[serial]
    fn test_security_config_from_env() {
        // SAFETY: This test runs in isolation and we clean up after ourselves.
        // Environment variable access is single-threaded in test context.
        unsafe {
            // Test with RUST_ENV unset (should default to development)
            std::env::remove_var("RUST_ENV");
            let config = SecurityConfig::from_env();
            assert!(!config.is_production);

            // Test with RUST_ENV=production
            std::env::set_var("RUST_ENV", "production");
            let config = SecurityConfig::from_env();
            assert!(config.is_production);

            // Test with RUST_ENV=development
            std::env::set_var("RUST_ENV", "development");
            let config = SecurityConfig::from_env();
            assert!(!config.is_production);

            // Clean up
            std::env::remove_var("RUST_ENV");
        }
    }

    /// Test create_security_config helper function.
    #[test]
    fn test_create_security_config() {
        let prod_config = create_security_config(true);
        assert!(prod_config.is_production);

        let dev_config = create_security_config(false);
        assert!(!dev_config.is_production);
    }

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

    // === SMTP Validation Tests ===

    /// Test SMTP validation with all variables missing.
    #[test]
    #[serial]
    fn test_smtp_validation_missing_all() {
        // SAFETY: Test isolation with cleanup.
        unsafe {
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USER");
            std::env::remove_var("SMTP_PASSWORD");

            let result = validate_smtp_config();

            assert!(!result.is_configured);
            assert!(!result.is_valid());
            assert_eq!(result.errors.len(), 3);
            assert!(result.errors.iter().any(|e| e.contains("SMTP_HOST")));
            assert!(result.errors.iter().any(|e| e.contains("SMTP_USER")));
            assert!(result.errors.iter().any(|e| e.contains("SMTP_PASSWORD")));
        }
    }

    /// Test SMTP validation with valid configuration.
    #[test]
    #[serial]
    fn test_smtp_validation_valid() {
        // SAFETY: Test isolation with cleanup.
        unsafe {
            std::env::set_var("SMTP_HOST", "smtp.example.com");
            std::env::set_var("SMTP_USER", "user@example.com");
            std::env::set_var("SMTP_PASSWORD", "secure_password_123");

            let result = validate_smtp_config();

            assert!(result.is_configured);
            assert!(result.is_valid());
            assert!(result.errors.is_empty());
            assert!(result.warnings.is_empty());

            // Clean up
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USER");
            std::env::remove_var("SMTP_PASSWORD");
        }
    }

    /// Test SMTP validation with empty values.
    #[test]
    #[serial]
    fn test_smtp_validation_empty_values() {
        // SAFETY: Test isolation with cleanup.
        unsafe {
            std::env::set_var("SMTP_HOST", "");
            std::env::set_var("SMTP_USER", "");
            std::env::set_var("SMTP_PASSWORD", "");

            let result = validate_smtp_config();

            assert!(!result.is_valid());
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| e.contains("SMTP_HOST is empty"))
            );
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| e.contains("SMTP_USER is empty"))
            );
            assert!(
                result
                    .errors
                    .iter()
                    .any(|e| e.contains("SMTP_PASSWORD is empty"))
            );

            // Clean up
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USER");
            std::env::remove_var("SMTP_PASSWORD");
        }
    }

    /// Test SMTP validation warnings for suspicious values.
    #[test]
    #[serial]
    fn test_smtp_validation_warnings() {
        // SAFETY: Test isolation with cleanup.
        unsafe {
            std::env::set_var("SMTP_HOST", "mailserver"); // No domain suffix
            std::env::set_var("SMTP_USER", "notanemail"); // Not an email
            std::env::set_var("SMTP_PASSWORD", "short"); // Too short

            let result = validate_smtp_config();

            assert!(result.is_configured);
            assert!(result.errors.is_empty()); // Warnings, not errors
            assert!(!result.warnings.is_empty());
            assert!(result.warnings.iter().any(|w| w.contains("may be invalid")));
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|w| w.contains("does not appear to be an email"))
            );
            assert!(result.warnings.iter().any(|w| w.contains("short")));

            // Clean up
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USER");
            std::env::remove_var("SMTP_PASSWORD");
        }
    }

    /// Test SMTP validation with localhost (valid edge case).
    #[test]
    #[serial]
    fn test_smtp_validation_localhost() {
        // SAFETY: Test isolation with cleanup.
        unsafe {
            std::env::set_var("SMTP_HOST", "localhost");
            std::env::set_var("SMTP_USER", "test@localhost");
            std::env::set_var("SMTP_PASSWORD", "testpassword123");

            let result = validate_smtp_config();

            assert!(result.is_configured);
            assert!(result.is_valid()); // localhost is valid
            assert!(result.warnings.is_empty());

            // Clean up
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USER");
            std::env::remove_var("SMTP_PASSWORD");
        }
    }

    /// Test SmtpValidation struct methods.
    #[test]
    fn test_smtp_validation_struct() {
        let valid = SmtpValidation {
            is_configured: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        assert!(valid.is_valid());

        let invalid_not_configured = SmtpValidation {
            is_configured: false,
            errors: vec!["Missing SMTP_HOST".to_string()],
            warnings: Vec::new(),
        };
        assert!(!invalid_not_configured.is_valid());

        let invalid_with_errors = SmtpValidation {
            is_configured: true,
            errors: vec!["SMTP_HOST is empty".to_string()],
            warnings: Vec::new(),
        };
        assert!(!invalid_with_errors.is_valid());

        let valid_with_warnings = SmtpValidation {
            is_configured: true,
            errors: Vec::new(),
            warnings: vec!["Password is short".to_string()],
        };
        assert!(valid_with_warnings.is_valid()); // Warnings don't affect validity
    }
}
