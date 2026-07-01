use std::time::Duration;

use tokio_retry::{strategy::ExponentialBackoff, Retry};
use tracing::{error, warn};

/// Configuration for retry behavior when invoking asynchronous operations.
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    /// The initial delay in milliseconds before the first retry attempt.
    pub initial_delay_millis: u64,
    /// The maximum delay in seconds between retry attempts.
    pub max_delay_secs: u64,
    /// The maximum number of retry attempts.
    pub max_retries: u32,
}

impl RetryConfig {
    /// Creates a new `RetryConfig` with specified delay and retry parameters.
    #[must_use]
    pub fn new(initial_delay_millis: u64, max_delay_secs: u64, max_retries: u32) -> Self {
        Self {
            initial_delay_millis,
            max_delay_secs,
            max_retries,
        }
    }

    fn strategy(&self) -> impl Iterator<Item = Duration> + Clone {
        ExponentialBackoff::from_millis(self.initial_delay_millis)
            .max_delay(Duration::from_secs(self.max_delay_secs))
            .take(self.max_retries as usize)
    }
}

impl Default for RetryConfig {
    /// Returns a default `RetryConfig` with `initial_delay_millis` of 50ms,
    /// `max_delay_secs` of 2s, and `max_retries` of 3.
    fn default() -> Self {
        Self {
            initial_delay_millis: 50,
            max_delay_secs: 2,
            max_retries: 3,
        }
    }
}

/// Execute an asynchronous operation with retry/backoff semantics.
///
/// `context` is included in log messages to provide call-site visibility.
pub async fn retry_async<F, Fut, T, E>(
    context: &str,
    config: RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let strategy = config.strategy();
    let result = Retry::spawn(strategy, || {
        let fut = operation();
        async move {
            match fut.await {
                Ok(value) => Ok(value),
                Err(err) => {
                    warn!(error = ?err, retry_context = context, "Operation failed; retrying");
                    Err(err)
                }
            }
        }
    })
    .await;

    if let Err(err) = &result {
        error!(
            error = ?err,
            retry_context = context,
            "Operation failed after exhausting retries"
        );
    }

    result
}

/// Escapes HTML metacharacters so untrusted text can be safely interpolated
/// into HTML output or SurrealDB query strings.
///
/// Canonical implementation shared by the `app` crate's server functions and
/// the `server` crate's validation module (previously copy-pasted in both).
#[must_use]
pub fn sanitize_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '/' => result.push_str("&#x2F;"),
            '`' => result.push_str("&#x60;"),
            _ => result.push(c),
        }
    }
    result
}

/// Returns `true` if every character in `s` is allowed in a URL slug:
/// ASCII alphanumerics, hyphens, and underscores.
#[must_use]
pub fn slug_chars_valid(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Returns `true` if every character in `s` is allowed in a tag:
/// ASCII alphanumerics, hyphens, underscores, and spaces.
#[must_use]
pub fn tag_chars_valid(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
}

/// Returns `true` if `slug` is non-empty, within `max_len` bytes, and contains
/// only slug-safe characters. Used as a guard before interpolating a slug into
/// a database query.
#[must_use]
pub fn is_valid_slug(slug: &str, max_len: usize) -> bool {
    !slug.is_empty() && slug.len() <= max_len && slug_chars_valid(slug)
}

/// Returns `true` if `tag` is non-empty, within `max_len` bytes, and contains
/// only tag-safe characters.
#[must_use]
pub fn is_valid_tag(tag: &str, max_len: usize) -> bool {
    !tag.is_empty() && tag.len() <= max_len && tag_chars_valid(tag)
}

/// Returns `true` if `email` is structurally valid: a single `@`, a 1-64 byte
/// local part, a dotted domain whose final label (TLD) is at least two
/// characters, and only standard email characters throughout.
///
/// This is the boolean kernel of the email rule; the `server` crate's
/// `validate_email` performs the same checks but returns granular,
/// user-facing error messages.
#[must_use]
pub fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return false;
    }
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }
    match domain_parts.last() {
        Some(tld) if tld.len() >= 2 => {}
        _ => return false,
    }
    let valid_local = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));
    let valid_domain = domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));
    valid_local && valid_domain
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    /// Tests that the `retry_async` function successfully completes after a few retries.
    fn succeeds_after_retries() {
        tokio_test::block_on(async {
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let result = retry_async("test_success", RetryConfig::default(), move || {
                let tracker = tracker.clone();
                async move {
                    let current = tracker.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err::<_, &'static str>("fail")
                    } else {
                        Ok::<_, &'static str>("ok")
                    }
                }
            })
            .await;

            assert_eq!(result.unwrap(), "ok");
            assert_eq!(attempts.load(Ordering::SeqCst), 3);
        });
    }

    #[test]
    /// Tests that the `retry_async` function returns an error after exhausting all retry attempts.
    fn returns_error_after_exhausting_retries() {
        tokio_test::block_on(async {
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let config = RetryConfig::default();
            let result: Result<(), &str> = retry_async("test_failure", config, move || {
                let tracker = tracker.clone();
                async move {
                    tracker.fetch_add(1, Ordering::SeqCst);
                    Err("nope")
                }
            })
            .await;

            assert!(result.is_err());
            assert_eq!(
                attempts.load(Ordering::SeqCst),
                config.max_retries as usize + 1
            );
        });
    }

    #[test]
    /// Tests that the `retry_async` function correctly applies a custom `RetryConfig`.
    fn honors_custom_config() {
        tokio_test::block_on(async {
            let config = RetryConfig::new(10, 1, 5);
            let attempts = Arc::new(AtomicUsize::new(0));
            let tracker = attempts.clone();

            let _ = retry_async("custom_config", config, move || {
                let tracker = tracker.clone();
                async move {
                    tracker.fetch_add(1, Ordering::SeqCst);
                    Err::<(), &str>("fail")
                }
            })
            .await;

            assert_eq!(
                attempts.load(Ordering::SeqCst),
                config.max_retries as usize + 1
            );
        });
    }

    #[test]
    fn sanitize_html_escapes_all_metacharacters() {
        assert_eq!(
            sanitize_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );
        assert_eq!(sanitize_html("a & b `c`"), "a &amp; b &#x60;c&#x60;");
        assert_eq!(sanitize_html("plain text"), "plain text");
    }

    #[test]
    fn is_valid_slug_accepts_safe_slugs() {
        assert!(is_valid_slug("hello-world", 200));
        assert!(is_valid_slug("my_post_123", 200));
        assert!(is_valid_slug("PostTitle", 200));
        assert!(is_valid_slug("a", 200));
        assert!(is_valid_slug("123", 200));
    }

    #[test]
    fn is_valid_slug_rejects_unsafe_slugs() {
        assert!(!is_valid_slug("", 200));
        assert!(!is_valid_slug("hello world", 200)); // spaces
        assert!(!is_valid_slug("hello\"world", 200)); // quotes
        assert!(!is_valid_slug("hello'world", 200)); // single quotes
        assert!(!is_valid_slug("hello;world", 200)); // semicolon
        assert!(!is_valid_slug("hello\nworld", 200)); // newline
        assert!(!is_valid_slug(&"a".repeat(201), 200)); // too long
    }

    #[test]
    fn is_valid_tag_accepts_safe_tags() {
        assert!(is_valid_tag("rust", 100));
        assert!(is_valid_tag("web-dev", 100));
        assert!(is_valid_tag("programming_tips", 100));
        assert!(is_valid_tag("machine learning", 100)); // spaces allowed in tags
    }

    #[test]
    fn is_valid_tag_rejects_unsafe_tags() {
        assert!(!is_valid_tag("", 100));
        assert!(!is_valid_tag("tag\"injection", 100)); // quotes
        assert!(!is_valid_tag("tag;drop", 100)); // semicolon
        assert!(!is_valid_tag("tag\ttab", 100)); // tab
        assert!(!is_valid_tag(&"a".repeat(101), 100)); // too long
    }

    #[test]
    fn is_valid_email_accepts_valid_addresses() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user+tag@sub.example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(is_valid_email("  User@Example.COM  ")); // trimmed
    }

    #[test]
    fn is_valid_email_rejects_invalid_addresses() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("invalid")); // no @
        assert!(!is_valid_email("user@@example.com")); // multiple @
        assert!(!is_valid_email("user@")); // no domain
        assert!(!is_valid_email("@example.com")); // no local part
        assert!(!is_valid_email("user@domain")); // no TLD
        assert!(!is_valid_email("user@example.c")); // single-char TLD
        assert!(!is_valid_email("user@.com")); // empty domain part
        assert!(!is_valid_email("user<script>@example.com")); // bad chars
    }
}
