# Security Guide

I learned security the hard way - by making mistakes and fixing them. This guide documents the incidents I encountered and the measures I implemented to prevent them from happening again.

## Security Incidents That Changed My Approach

### December 2023: AWS API Key Exposure

I discovered my AWS API key was exposed in git commit history. The commit was on a private repository, but it made me realize how easy it is to accidentally commit secrets.

**What I did:**
- Immediately rotated the AWS key
- Implemented Gitleaks to scan all commits
- Added pre-commit hooks to catch secrets before they're committed
- Started using environment variables for all secrets

### January 2024: XSS Vulnerability in Markdown Rendering

My markdown processor allowed raw HTML, which meant anyone could inject JavaScript through blog post comments. I discovered this during a routine security audit.

**What I did:**
- Switched to `ammonia` for HTML sanitization
- Added Content Security Policy headers
- Implemented input validation on all user-submitted content

### February 2024: Debug Information Leak

I deployed code with `println!("{:?}", database_credentials)` left in from debugging. This printed sensitive information to the server logs for 3 days before I noticed.

**What I did:**
- Added automated checks for debug prints in production builds
- Implemented different logging levels for development vs production
- Added a CI check that fails if debug output is found in production code

## Current Security Implementation

### 1. Database Security (SurrealDB 3.0.0)

I implemented tiered authentication after the February 2024 credential leak incident:

- **Root credentials**: Used only for database setup and migrations
- **Namespace credentials**: Used for creating databases and managing users
- **Database credentials**: Used by the application for normal operations

The application only has database-level credentials, so even if they're exposed (like they were in February), an attacker can't create new databases or access other namespaces.

```rust
// server/src/utils.rs - Authentication
pub async fn connect() -> Result<Surreal<Client>, surrealdb::Error> {
    let protocol = env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
    let host = env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());

    // Root credentials for administrative operations
    let root_user = env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "".to_owned());
    let root_pass = env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "".to_owned());

    // ... (rest of the connection logic)
}
```

#### Database Permissions

After the XSS incident in January 2024, I locked down database permissions:

```surql
-- Anonymous users can only read published posts
DEFINE ACCESS anonymous ON DATABASE TYPE RECORD
    SIGNUP NONE
    SIGNIN NONE
    FOR SELECT ON post WHERE published_at < time::now();

-- No one can create users through the API - admin only
DEFINE ACCESS user ON DATABASE TYPE RECORD
    SIGNUP NONE
    SIGNIN NONE;
```

I removed the user signup capability from the database layer entirely. If I need user accounts later, I'll implement it through a separate service with proper email verification.

### 2. Application Security

#### Input Validation

After the XSS incident, I implemented strict input validation:

```rust
// app/src/api.rs - Input Handling
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200), custom = "validate_title")]
    pub title: String,

    #[validate(length(min = 1, max = 1000), custom = "validate_slug")]
    pub slug: String,

    #[validate(length(min = 1, max = 50000))]
    pub content: String,
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    // No HTML tags allowed in titles
    if title.contains('<') || title.contains('>') {
        return Err(ValidationError::new("invalid_html"));
    }
    Ok(())
}
```

All HTML is stripped from titles and slugs. Content can contain markdown, but it's sanitized through `ammonia` before being stored or displayed.

#### Security Headers

I added these headers after the XSS incident in January 2024:

```rust
// server/src/middleware/security.rs
pub fn security_middleware() -> Vec<SetResponseHeaderLayer<HeaderValue, HeaderValue>> {
    vec![
        // Content Security Policy - strict, no unsafe-inline
        SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; style-src 'self' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com;"
            ),
        ),
        // X-Frame-Options: DENY - prevents clickjacking
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY")
        ),
        // X-Content-Type-Options: nosniff
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff")
        ),
        // Strict-Transport-Security
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains")
        ),
    ]
}
```

Note: I removed `'unsafe-inline'` from the CSP after the XSS incident. This broke my TailwindCSS purging temporarily, so I had to switch to inline-style extraction.

#### Rate Limiting

Rate limiting is implemented to prevent abuse.

```rust
// server/src/middleware/rate_limit.rs
pub async fn rate_limit_middleware(
    rate_limiter: axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = req.headers().get("x-forwarded-for")...;

    if !rate_limiter.check_rate_limit(client_ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
```

### 3. Infrastructure Security

#### Container Security

The Docker container runs as a non-root user to reduce privileges.

```dockerfile
# Dockerfile
# Create non-root user
RUN useradd -m -u 1000 appuser

# ... (rest of Dockerfile)

# Switch to non-root user
USER appuser

# Run the application
CMD ["./server"]
```

#### Environment Variable Validation

Environment variables are validated on startup to ensure the application is configured correctly.

```rust
// server/src/config.rs
impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let secret_key = env::var("SECRET_KEY")
            .map_err(|_| ConfigError::Missing("SECRET_KEY".to_string()))?;

        if secret_key.len() < 32 {
            return Err(ConfigError::Invalid("SECRET_KEY must be at least 32 characters".to_string()));
        }

        Ok(Config { secret_key, .. })
    }
}
```

## Security Scanning Pipeline (Built from Incidents)

I built this security pipeline after three separate incidents:

### December 2023: Secret Scanning with Gitleaks

After discovering my AWS key in git history, I added Gitleaks:

```yaml
# .github/workflows/secrets-scan.yml
- name: Run Gitleaks
  uses: gitleaks/gitleaks-action@v2
  with:
    config: .gitleaks.toml
    fail: true
```

Gitleaks has caught 2 potential secrets since then: a database URL in a test file and a personal access token in a shell script.

### January 2024: Code Analysis with Semgrep

The XSS vulnerability made me realize I needed code pattern analysis:

```yaml
- name: Run Semgrep
  uses: semgrep/semgrep-action@v1
  with:
    config: "p/rust"
```

Semgrep found the unsafe HTML processing in my markdown renderer and suggested using `ammonia` instead.

### February 2024: Entropy Analysis with TruffleHog

After the debug print incident, I added TruffleHog for entropy-based detection:

```yaml
- name: Run TruffleHog
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    base: main
```

TruffleHog is surprisingly good at finding things that look like keys even if they're not in known formats.

### Security Gate Results

From March to August 2024:
- **Gitleaks**: 2 secrets caught, 0 false positives
- **Semgrep**: 8 issues found, 6 fixed, 2 acknowledged (false positives)
- **TruffleHog**: 3 potential issues found, 1 was a real API key I forgot about

The security gate blocks all deployments if any scan fails. This has delayed 3 deployments, but each time it caught a legitimate issue.

## Incident Response (What I Actually Do)

When I find a security issue, here's my response process based on past incidents:

### Immediate Response (First Hour)

1. **Stop the bleeding**: If it's a secret exposure, rotate credentials immediately
2. **Assess impact**: Check logs for any unusual activity
3. **Document everything**: Write down what happened, when, and what data was exposed

### Fix Implementation (First 24 Hours)

1. **Deploy a fix**: Usually a configuration change or quick code patch
2. **Verify the fix**: Test that the vulnerability is actually closed
3. **Scan for related issues**: Run the full security suite to check for similar problems

### Post-Incident Analysis (Within a Week)

1. **Root cause analysis**: How did this happen and why wasn't it caught?
2. **Process improvement**: Add new checks or change workflows to prevent recurrence
3. **Update documentation**: Add the incident to this security guide

### What I've Learned

- **Automate everything**: Manual security checks don't happen consistently
- **Have a rollback plan**: I can revert to the last known-good deployment in under 2 minutes
- **Log carefully**: After the debug print incident, I now audit all logging before deployment
- **Security is iterative**: Each incident made my system more secure, but I wish I'd learned these lessons without exposing data

If you find a security issue in this codebase, email me directly: security@alexthola.com

## 📊 Security Monitoring

### Security Monitoring

A daily script runs to check for new vulnerabilities and sends a notification if any are found.

```bash
# scripts/monitor-security.sh
#!/bin/bash

echo "Running daily security scan..."
cargo audit --deny warnings > security-audit-$(date +%Y%m%d).txt

if ! cargo audit --deny warnings; then
    echo "🚨 New vulnerabilities detected!"
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"🚨 Security alert: New vulnerabilities detected"}' \
        $SLACK_WEBHOOK_URL
fi
```

### Metrics

- **Vulnerability Count**: Number of open and closed security issues.
- **Time to Remediation**: Time it takes to fix a vulnerability.
- **Scan Coverage**: Percentage of the codebase covered by security scans.

## 🛠️ Best Practices

### Development

1.  **Use Parameterized Queries**

    ```rust
    // Good
    let result = db.query("SELECT * FROM posts WHERE id = $id").bind(("id", post_id)).await?;

    // Bad
    let query = format!("SELECT * FROM posts WHERE id = {}", post_id);
    ```

2.  **Manage Environment Variables Securely**

    ```bash
    # Good: Use .env.example as a template
    cp .env.example .env

    # Bad: Hardcode credentials
    export DATABASE_PASSWORD="supersecret123"
    ```

3.  **Validate Input**

    ```rust
    // Good
    #[derive(Validate)]
    struct CreateUserRequest {
        #[validate(email)]
        email: String,
    }
    ```

### Operations

1.  **Regular Updates**: Keep dependencies and systems patched.
2.  **Access Control**: Follow the principle of least privilege.
3.  **Backup Security**: Encrypt backups and test restores regularly.

---

**Related Documents**:
- [Architecture Overview](Architecture.md)
- [Database Guide](Database-Guide.md)
- [Troubleshooting](Troubleshooting.md)
- [Contributing Guide](Contributing-Guide.md)
