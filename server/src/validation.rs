//! Input validation and sanitization middleware for the blog application.
//!
//! This module provides comprehensive input validation and sanitization functions
//! to prevent XSS, injection attacks, and other security vulnerabilities.
//!
//! # Security Features
//!
//! - HTML entity encoding for string outputs
//! - Length validation helpers
//! - Character whitelist validation
//! - Email format validation
//! - Message sanitization with controlled HTML

use std::fmt;

/// Validation error types for input validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Input exceeds maximum allowed length.
    TooLong {
        /// Maximum allowed length.
        max: usize,
        /// Actual length of input.
        actual: usize,
    },
    /// Input is shorter than minimum required length.
    TooShort {
        /// Minimum required length.
        min: usize,
        /// Actual length of input.
        actual: usize,
    },
    /// Input contains characters not in the allowed set.
    InvalidCharacters {
        /// Description of allowed characters.
        allowed: String,
    },
    /// Input does not match expected format.
    InvalidFormat {
        /// Description of expected format.
        expected: String,
    },
    /// Input is empty when a value is required.
    Empty,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::TooLong { max, actual } => {
                write!(
                    f,
                    "Input too long: maximum {} characters, got {}",
                    max, actual
                )
            }
            ValidationError::TooShort { min, actual } => {
                write!(
                    f,
                    "Input too short: minimum {} characters, got {}",
                    min, actual
                )
            }
            ValidationError::InvalidCharacters { allowed } => {
                write!(f, "Invalid characters: only {} are allowed", allowed)
            }
            ValidationError::InvalidFormat { expected } => {
                write!(f, "Invalid format: expected {}", expected)
            }
            ValidationError::Empty => write!(f, "Input cannot be empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Sanitizes a string by escaping HTML entities to prevent XSS attacks.
///
/// This function replaces dangerous HTML characters with their entity equivalents:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&#x27;`
/// - `/` -> `&#x2F;`
/// - `` ` `` -> `&#x60;`
///
/// # Arguments
///
/// * `input` - The string to sanitize.
///
/// # Returns
///
/// A new string with all HTML entities properly escaped.
///
/// # Examples
///
/// ```
/// use server::validation::sanitize_html;
///
/// let input = "<script>alert('xss')</script>";
/// let sanitized = sanitize_html(input);
/// assert_eq!(sanitized, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;");
/// ```
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

/// Validates and sanitizes a slug.
///
/// A valid slug contains only alphanumeric characters (a-z, A-Z, 0-9),
/// hyphens (-), and underscores (_). The slug is trimmed of whitespace
/// and converted to lowercase for consistency.
///
/// # Arguments
///
/// * `input` - The slug string to validate.
/// * `max_len` - Maximum allowed length for the slug.
///
/// # Returns
///
/// A `Result` containing the sanitized slug on success, or a `ValidationError` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The input is empty
/// - The input exceeds `max_len` characters
/// - The input contains invalid characters
///
/// # Examples
///
/// ```
/// use server::validation::validate_slug;
///
/// assert!(validate_slug("hello-world", 200).is_ok());
/// assert!(validate_slug("my_post_123", 200).is_ok());
/// assert!(validate_slug("hello world", 200).is_err()); // spaces not allowed
/// ```
pub fn validate_slug(input: &str, max_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: trimmed.len(),
        });
    }

    // Validate characters: only alphanumeric, hyphens, and underscores
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ValidationError::InvalidCharacters {
            allowed: "alphanumeric characters, hyphens (-), and underscores (_)".to_string(),
        });
    }

    Ok(trimmed.to_lowercase())
}

/// Validates and sanitizes a tag.
///
/// A valid tag contains only alphanumeric characters (a-z, A-Z, 0-9),
/// hyphens (-), underscores (_), and spaces. The tag is trimmed and
/// normalized (multiple spaces collapsed to single spaces).
///
/// # Arguments
///
/// * `input` - The tag string to validate.
/// * `max_len` - Maximum allowed length for the tag.
///
/// # Returns
///
/// A `Result` containing the sanitized tag on success, or a `ValidationError` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The input is empty
/// - The input exceeds `max_len` characters
/// - The input contains invalid characters
///
/// # Examples
///
/// ```
/// use server::validation::validate_tag;
///
/// assert!(validate_tag("machine learning", 100).is_ok());
/// assert!(validate_tag("web-dev", 100).is_ok());
/// assert!(validate_tag("rust<script>", 100).is_err()); // HTML not allowed
/// ```
pub fn validate_tag(input: &str, max_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: trimmed.len(),
        });
    }

    // Validate characters: only alphanumeric, hyphens, underscores, and spaces
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err(ValidationError::InvalidCharacters {
            allowed: "alphanumeric characters, hyphens (-), underscores (_), and spaces"
                .to_string(),
        });
    }

    // Normalize whitespace (collapse multiple spaces to single space)
    let normalized: String = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

    Ok(normalized)
}

/// Validates an email address format.
///
/// This function performs a reasonable validation of email format without
/// being overly strict. It checks for:
/// - Non-empty local and domain parts
/// - Presence of exactly one `@` symbol
/// - Valid domain with at least one `.`
/// - Maximum length of 254 characters (RFC 5321)
///
/// Note: This is not a complete RFC 5322 validation but covers common cases
/// and prevents obvious injection attempts.
///
/// # Arguments
///
/// * `input` - The email string to validate.
///
/// # Returns
///
/// A `Result` containing the validated email (trimmed and lowercased) on success,
/// or a `ValidationError` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The input is empty
/// - The input exceeds 254 characters
/// - The input does not match a valid email format
///
/// # Examples
///
/// ```
/// use server::validation::validate_email;
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("user+tag@sub.example.com").is_ok());
/// assert!(validate_email("invalid").is_err());
/// assert!(validate_email("no@domain").is_err());
/// ```
pub fn validate_email(input: &str) -> Result<String, ValidationError> {
    const MAX_EMAIL_LEN: usize = 254;

    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() > MAX_EMAIL_LEN {
        return Err(ValidationError::TooLong {
            max: MAX_EMAIL_LEN,
            actual: trimmed.len(),
        });
    }

    // Basic email format validation
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError::InvalidFormat {
            expected: "valid email address (user@domain.com)".to_string(),
        });
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part validation
    if local.is_empty() || local.len() > 64 {
        return Err(ValidationError::InvalidFormat {
            expected: "valid email address with local part (1-64 characters)".to_string(),
        });
    }

    // Domain validation
    if domain.is_empty() || !domain.contains('.') {
        return Err(ValidationError::InvalidFormat {
            expected: "valid email address with domain (e.g., example.com)".to_string(),
        });
    }

    // Check domain parts
    let domain_parts: Vec<&str> = domain.split('.').collect();
    for part in &domain_parts {
        if part.is_empty() {
            return Err(ValidationError::InvalidFormat {
                expected: "valid email address with proper domain format".to_string(),
            });
        }
    }

    // Check TLD exists (at least 2 characters)
    if let Some(tld) = domain_parts.last()
        && tld.len() < 2
    {
        return Err(ValidationError::InvalidFormat {
            expected: "valid email address with valid TLD (at least 2 characters)".to_string(),
        });
    }

    // Validate allowed characters in email
    // Local part: alphanumeric plus . _ % + -
    // Domain: alphanumeric plus . -
    let valid_local = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));

    let valid_domain = domain
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));

    if !valid_local || !valid_domain {
        return Err(ValidationError::InvalidCharacters {
            allowed: "alphanumeric characters and standard email special characters".to_string(),
        });
    }

    Ok(trimmed.to_lowercase())
}

/// Sanitizes a contact form message by escaping dangerous patterns.
///
/// This function:
/// - Escapes HTML entities to prevent XSS
/// - Validates length constraints
/// - Trims excessive whitespace
///
/// Unlike full HTML sanitization, this preserves newlines for readability
/// while escaping all HTML to prevent injection.
///
/// # Arguments
///
/// * `input` - The message string to sanitize.
/// * `max_len` - Maximum allowed length for the message.
///
/// # Returns
///
/// A `Result` containing the sanitized message on success, or a `ValidationError` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The input is empty
/// - The input exceeds `max_len` characters
///
/// # Examples
///
/// ```
/// use server::validation::sanitize_message;
///
/// let message = "Hello, <script>alert('xss')</script>";
/// let result = sanitize_message(message, 5000);
/// assert!(result.is_ok());
/// assert!(!result.unwrap().contains("<script>"));
/// ```
pub fn sanitize_message(input: &str, max_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: trimmed.len(),
        });
    }

    // Escape all HTML entities while preserving newlines
    let sanitized = sanitize_html(trimmed);

    Ok(sanitized)
}

/// Validates and sanitizes a name (e.g., contact form name field).
///
/// A valid name:
/// - Is not empty
/// - Does not exceed the maximum length
/// - Has all HTML entities escaped
///
/// # Arguments
///
/// * `input` - The name string to validate.
/// * `max_len` - Maximum allowed length for the name.
///
/// # Returns
///
/// A `Result` containing the sanitized name on success, or a `ValidationError` on failure.
///
/// # Errors
///
/// Returns an error if:
/// - The input is empty
/// - The input exceeds `max_len` characters
///
/// # Examples
///
/// ```
/// use server::validation::validate_name;
///
/// assert!(validate_name("John Doe", 100).is_ok());
/// assert!(validate_name("<script>", 100).is_ok()); // Returns sanitized version
/// assert!(validate_name("", 100).is_err());
/// ```
pub fn validate_name(input: &str, max_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: trimmed.len(),
        });
    }

    // Sanitize HTML but allow unicode characters (international names)
    Ok(sanitize_html(trimmed))
}

/// Validates that a string does not exceed a maximum length.
///
/// This is a simple helper for length validation.
///
/// # Arguments
///
/// * `input` - The string to validate.
/// * `max_len` - Maximum allowed length.
///
/// # Returns
///
/// A `Result` containing the trimmed input on success, or a `ValidationError` on failure.
pub fn validate_length(input: &str, max_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.len() > max_len {
        return Err(ValidationError::TooLong {
            max: max_len,
            actual: trimmed.len(),
        });
    }

    Ok(trimmed.to_string())
}

/// Validates that a string has a minimum length.
///
/// This is a helper for minimum length validation.
///
/// # Arguments
///
/// * `input` - The string to validate.
/// * `min_len` - Minimum required length.
///
/// # Returns
///
/// A `Result` containing the trimmed input on success, or a `ValidationError` on failure.
pub fn validate_min_length(input: &str, min_len: usize) -> Result<String, ValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() && min_len > 0 {
        return Err(ValidationError::Empty);
    }

    if trimmed.len() < min_len {
        return Err(ValidationError::TooShort {
            min: min_len,
            actual: trimmed.len(),
        });
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // === sanitize_html tests ===

    #[test]
    fn test_sanitize_html_basic() {
        assert_eq!(sanitize_html("hello"), "hello");
        assert_eq!(sanitize_html(""), "");
        assert_eq!(sanitize_html("Hello World"), "Hello World");
    }

    #[test]
    fn test_sanitize_html_escapes_ampersand() {
        assert_eq!(sanitize_html("Tom & Jerry"), "Tom &amp; Jerry");
        assert_eq!(sanitize_html("a&b&c"), "a&amp;b&amp;c");
    }

    #[test]
    fn test_sanitize_html_escapes_angle_brackets() {
        assert_eq!(sanitize_html("<script>"), "&lt;script&gt;");
        assert_eq!(sanitize_html("a < b > c"), "a &lt; b &gt; c");
    }

    #[test]
    fn test_sanitize_html_escapes_quotes() {
        assert_eq!(sanitize_html(r#""quoted""#), "&quot;quoted&quot;");
        assert_eq!(sanitize_html("it's"), "it&#x27;s");
    }

    #[test]
    fn test_sanitize_html_escapes_slash_and_backtick() {
        assert_eq!(sanitize_html("a/b"), "a&#x2F;b");
        assert_eq!(sanitize_html("`code`"), "&#x60;code&#x60;");
    }

    #[test]
    fn test_sanitize_html_xss_prevention() {
        let xss = r#"<script>alert("XSS")</script>"#;
        let sanitized = sanitize_html(xss);
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
        assert_eq!(
            sanitized,
            "&lt;script&gt;alert(&quot;XSS&quot;)&lt;&#x2F;script&gt;"
        );
    }

    #[test]
    fn test_sanitize_html_image_onerror() {
        let xss = r#"<img src="x" onerror="alert('xss')">"#;
        let sanitized = sanitize_html(xss);
        assert!(!sanitized.contains('<'));
        assert!(sanitized.contains("&lt;img"));
    }

    #[test]
    fn test_sanitize_html_preserves_unicode() {
        assert_eq!(sanitize_html("Hello 世界"), "Hello 世界");
        assert_eq!(sanitize_html("Café résumé"), "Café résumé");
        assert_eq!(sanitize_html("emoji: 🚀"), "emoji: 🚀");
    }

    // === validate_slug tests ===

    #[test]
    fn test_validate_slug_valid() {
        assert_eq!(
            validate_slug("hello-world", 200).unwrap(),
            "hello-world".to_string()
        );
        assert_eq!(
            validate_slug("my_post_123", 200).unwrap(),
            "my_post_123".to_string()
        );
        assert_eq!(
            validate_slug("PostTitle", 200).unwrap(),
            "posttitle".to_string()
        );
        assert_eq!(validate_slug("a", 200).unwrap(), "a".to_string());
        assert_eq!(validate_slug("123", 200).unwrap(), "123".to_string());
    }

    #[test]
    fn test_validate_slug_trims_whitespace() {
        assert_eq!(
            validate_slug("  hello-world  ", 200).unwrap(),
            "hello-world".to_string()
        );
    }

    #[test]
    fn test_validate_slug_lowercases() {
        assert_eq!(
            validate_slug("UPPERCASE", 200).unwrap(),
            "uppercase".to_string()
        );
        assert_eq!(
            validate_slug("MixedCase", 200).unwrap(),
            "mixedcase".to_string()
        );
    }

    #[test]
    fn test_validate_slug_empty() {
        assert_eq!(validate_slug("", 200), Err(ValidationError::Empty));
        assert_eq!(validate_slug("   ", 200), Err(ValidationError::Empty));
    }

    #[test]
    fn test_validate_slug_too_long() {
        let long_slug = "a".repeat(201);
        assert!(matches!(
            validate_slug(&long_slug, 200),
            Err(ValidationError::TooLong {
                max: 200,
                actual: 201
            })
        ));
    }

    #[test]
    fn test_validate_slug_invalid_characters() {
        assert!(matches!(
            validate_slug("hello world", 200),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_slug("hello@world", 200),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_slug("hello'world", 200),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_slug("hello\"world", 200),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_slug("hello;world", 200),
            Err(ValidationError::InvalidCharacters { .. })
        ));
    }

    #[test]
    fn test_validate_slug_sql_injection_prevention() {
        assert!(validate_slug("'; DROP TABLE posts; --", 200).is_err());
        assert!(validate_slug("1 OR 1=1", 200).is_err());
    }

    // === validate_tag tests ===

    #[test]
    fn test_validate_tag_valid() {
        assert_eq!(validate_tag("rust", 100).unwrap(), "rust".to_string());
        assert_eq!(validate_tag("web-dev", 100).unwrap(), "web-dev".to_string());
        assert_eq!(
            validate_tag("programming_tips", 100).unwrap(),
            "programming_tips".to_string()
        );
        assert_eq!(
            validate_tag("machine learning", 100).unwrap(),
            "machine learning".to_string()
        );
    }

    #[test]
    fn test_validate_tag_normalizes_whitespace() {
        assert_eq!(
            validate_tag("  machine   learning  ", 100).unwrap(),
            "machine learning".to_string()
        );
    }

    #[test]
    fn test_validate_tag_empty() {
        assert_eq!(validate_tag("", 100), Err(ValidationError::Empty));
        assert_eq!(validate_tag("   ", 100), Err(ValidationError::Empty));
    }

    #[test]
    fn test_validate_tag_too_long() {
        let long_tag = "a".repeat(101);
        assert!(matches!(
            validate_tag(&long_tag, 100),
            Err(ValidationError::TooLong {
                max: 100,
                actual: 101
            })
        ));
    }

    #[test]
    fn test_validate_tag_invalid_characters() {
        assert!(matches!(
            validate_tag("tag<script>", 100),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_tag("tag;injection", 100),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_tag("tag\ttab", 100),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_tag("tag\nnewline", 100),
            Err(ValidationError::InvalidCharacters { .. })
        ));
    }

    // === validate_email tests ===

    #[test]
    fn test_validate_email_valid() {
        assert_eq!(
            validate_email("user@example.com").unwrap(),
            "user@example.com".to_string()
        );
        assert_eq!(
            validate_email("user+tag@example.com").unwrap(),
            "user+tag@example.com".to_string()
        );
        assert_eq!(
            validate_email("user.name@sub.example.com").unwrap(),
            "user.name@sub.example.com".to_string()
        );
        assert_eq!(
            validate_email("USER@EXAMPLE.COM").unwrap(),
            "user@example.com".to_string()
        );
    }

    #[test]
    fn test_validate_email_trims_and_lowercases() {
        assert_eq!(
            validate_email("  User@Example.COM  ").unwrap(),
            "user@example.com".to_string()
        );
    }

    #[test]
    fn test_validate_email_empty() {
        assert_eq!(validate_email(""), Err(ValidationError::Empty));
        assert_eq!(validate_email("   "), Err(ValidationError::Empty));
    }

    #[test]
    fn test_validate_email_too_long() {
        // Create an email that exceeds the 254 character limit
        let long_local = "a".repeat(250);
        let long_email = format!("{}@example.com", long_local);
        assert!(long_email.len() > 254); // Sanity check
        assert!(matches!(
            validate_email(&long_email),
            Err(ValidationError::TooLong { max: 254, .. })
        ));
    }

    #[test]
    fn test_validate_email_invalid_format() {
        // No @
        assert!(matches!(
            validate_email("userexample.com"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // Multiple @
        assert!(matches!(
            validate_email("user@@example.com"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // No domain
        assert!(matches!(
            validate_email("user@"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // No local part
        assert!(matches!(
            validate_email("@example.com"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // No TLD
        assert!(matches!(
            validate_email("user@example"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // Single char TLD
        assert!(matches!(
            validate_email("user@example.c"),
            Err(ValidationError::InvalidFormat { .. })
        ));

        // Empty domain part
        assert!(matches!(
            validate_email("user@.com"),
            Err(ValidationError::InvalidFormat { .. })
        ));
    }

    #[test]
    fn test_validate_email_invalid_characters() {
        assert!(matches!(
            validate_email("user<script>@example.com"),
            Err(ValidationError::InvalidCharacters { .. })
        ));
        assert!(matches!(
            validate_email("user@exam ple.com"),
            Err(ValidationError::InvalidCharacters { .. })
        ));
    }

    // === sanitize_message tests ===

    #[test]
    fn test_sanitize_message_valid() {
        let result = sanitize_message("Hello, this is a test message.", 5000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, this is a test message.");
    }

    #[test]
    fn test_sanitize_message_escapes_html() {
        let result = sanitize_message("<script>alert('xss')</script>", 5000);
        assert!(result.is_ok());
        let sanitized = result.unwrap();
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_sanitize_message_preserves_newlines() {
        let result = sanitize_message("Line 1\nLine 2\nLine 3", 5000);
        assert!(result.is_ok());
        assert!(result.unwrap().contains('\n'));
    }

    #[test]
    fn test_sanitize_message_empty() {
        assert_eq!(sanitize_message("", 5000), Err(ValidationError::Empty));
        assert_eq!(sanitize_message("   ", 5000), Err(ValidationError::Empty));
    }

    #[test]
    fn test_sanitize_message_too_long() {
        let long_message = "a".repeat(5001);
        assert!(matches!(
            sanitize_message(&long_message, 5000),
            Err(ValidationError::TooLong {
                max: 5000,
                actual: 5001
            })
        ));
    }

    #[test]
    fn test_sanitize_message_unicode() {
        let result = sanitize_message("Hello 世界! 🚀 Привет!", 5000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello 世界! 🚀 Привет!");
    }

    // === validate_name tests ===

    #[test]
    fn test_validate_name_valid() {
        assert_eq!(
            validate_name("John Doe", 100).unwrap(),
            "John Doe".to_string()
        );
        assert_eq!(
            validate_name("Marie-Claire O'Brien", 100).unwrap(),
            "Marie-Claire O&#x27;Brien".to_string()
        );
    }

    #[test]
    fn test_validate_name_unicode() {
        assert_eq!(
            validate_name("田中太郎", 100).unwrap(),
            "田中太郎".to_string()
        );
        assert_eq!(validate_name("Müller", 100).unwrap(), "Müller".to_string());
    }

    #[test]
    fn test_validate_name_sanitizes_html() {
        let result = validate_name("<b>Bold</b>", 100).unwrap();
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
    }

    #[test]
    fn test_validate_name_empty() {
        assert_eq!(validate_name("", 100), Err(ValidationError::Empty));
        assert_eq!(validate_name("   ", 100), Err(ValidationError::Empty));
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(101);
        assert!(matches!(
            validate_name(&long_name, 100),
            Err(ValidationError::TooLong {
                max: 100,
                actual: 101
            })
        ));
    }

    // === validate_length tests ===

    #[test]
    fn test_validate_length_valid() {
        assert_eq!(validate_length("hello", 10).unwrap(), "hello".to_string());
        assert_eq!(
            validate_length("  hello  ", 10).unwrap(),
            "hello".to_string()
        );
    }

    #[test]
    fn test_validate_length_too_long() {
        assert!(matches!(
            validate_length("hello world", 5),
            Err(ValidationError::TooLong { max: 5, actual: 11 })
        ));
    }

    // === validate_min_length tests ===

    #[test]
    fn test_validate_min_length_valid() {
        assert_eq!(
            validate_min_length("hello", 3).unwrap(),
            "hello".to_string()
        );
        assert_eq!(
            validate_min_length("hello", 5).unwrap(),
            "hello".to_string()
        );
    }

    #[test]
    fn test_validate_min_length_empty() {
        assert_eq!(validate_min_length("", 1), Err(ValidationError::Empty));
        assert_eq!(validate_min_length("   ", 1), Err(ValidationError::Empty));
    }

    #[test]
    fn test_validate_min_length_too_short() {
        assert!(matches!(
            validate_min_length("hi", 5),
            Err(ValidationError::TooShort { min: 5, actual: 2 })
        ));
    }

    // === ValidationError Display tests ===

    #[test]
    fn test_validation_error_display() {
        assert_eq!(
            ValidationError::TooLong {
                max: 10,
                actual: 15
            }
            .to_string(),
            "Input too long: maximum 10 characters, got 15"
        );
        assert_eq!(
            ValidationError::TooShort { min: 5, actual: 2 }.to_string(),
            "Input too short: minimum 5 characters, got 2"
        );
        assert_eq!(
            ValidationError::InvalidCharacters {
                allowed: "letters".to_string()
            }
            .to_string(),
            "Invalid characters: only letters are allowed"
        );
        assert_eq!(
            ValidationError::InvalidFormat {
                expected: "email".to_string()
            }
            .to_string(),
            "Invalid format: expected email"
        );
        assert_eq!(ValidationError::Empty.to_string(), "Input cannot be empty");
    }

    // === Integration/Edge case tests ===

    #[test]
    fn test_combined_xss_attempts() {
        // Various XSS payloads should all be safely escaped
        let payloads = vec![
            r#"<img src=x onerror=alert(1)>"#,
            r#"<svg onload=alert(1)>"#,
            r#"javascript:alert(1)"#,
            r#"<a href="javascript:alert(1)">click</a>"#,
            r#"<iframe src="javascript:alert(1)">"#,
            r#"<body onload=alert(1)>"#,
            r#"<input onfocus=alert(1) autofocus>"#,
        ];

        for payload in payloads {
            let sanitized = sanitize_html(payload);
            assert!(
                !sanitized.contains('<'),
                "Payload {} was not sanitized",
                payload
            );
            assert!(
                !sanitized.contains('>'),
                "Payload {} was not sanitized",
                payload
            );
        }
    }

    #[test]
    fn test_sql_injection_prevention_in_slug() {
        let injections = vec![
            "'; DROP TABLE posts; --",
            "1 OR 1=1",
            "1; DELETE FROM users",
            "' OR '1'='1",
            "admin'--",
            "1 UNION SELECT * FROM users",
        ];

        for injection in injections {
            let result = validate_slug(injection, 200);
            assert!(
                result.is_err(),
                "SQL injection '{}' should be rejected",
                injection
            );
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // Exact max length
        let exact_slug = "a".repeat(200);
        assert!(validate_slug(&exact_slug, 200).is_ok());

        // One over max length
        let over_slug = "a".repeat(201);
        assert!(validate_slug(&over_slug, 200).is_err());

        // Exact max email length
        let domain = "example.com";
        let local_len = 254 - domain.len() - 1; // -1 for @
        let exact_email = format!("{}@{}", "a".repeat(local_len), domain);
        // This will fail local part validation (max 64) but tests the boundary
        assert!(validate_email(&exact_email).is_err());

        // Valid max local part
        let valid_email = format!("{}@example.com", "a".repeat(64));
        assert!(validate_email(&valid_email).is_ok());
    }
}
