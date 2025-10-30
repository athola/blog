# Security Guide

This security guide covers the implementation, practices, and policies that ensure the blog engine maintains enterprise-grade security standards.

## 🛡️ Security Architecture Overview

### Multi-Layer Security Approach

The blog engine implements defense-in-depth with security controls at multiple layers:

```
┌─────────────────────────────────────────────────────────┐
│                    Security Layers                      │
├─────────────────────────────────────────────────────────┤
│ 1. Infrastructure Security                                │
│    - Container security (non-root execution)              │
│    - Network security (HTTPS, firewalls)                 │
│    - Secrets management (environment variables)           │
├─────────────────────────────────────────────────────────┤
│ 2. Application Security                                   │
│    - Input validation and sanitization                    │
│    - Authentication and authorization                     │
│    - Security headers and CSP                             │
│    - Rate limiting and abuse prevention                  │
├─────────────────────────────────────────────────────────┤
│ 3. Data Security                                          │
│    - Database permissions (SurrealDB auth)                │
│    - Encrypted connections (TLS)                          │
│    - Data validation and type safety                      │
│    - Audit logging and monitoring                         │
├─────────────────────────────────────────────────────────┤
│ 4. Development Security                                   │
│    - Multi-tool security scanning                         │
│    - Dependency vulnerability management                 │
│    - Secure coding practices                              │
│    - Automated security gates in CI/CD                    │
└─────────────────────────────────────────────────────────┘
```

## 🔐 Implementation Details

### 1. Database Security (SurrealDB 3.0.0)

#### Multi-Level Authentication
```rust
// server/src/utils.rs - Enhanced authentication
pub async fn connect() -> Result<Surreal<Client>, surrealdb::Error> {
    let protocol = env::var("SURREAL_PROTOCOL").unwrap_or_else(|_| "http".to_owned());
    let host = env::var("SURREAL_HOST").unwrap_or_else(|_| "127.0.0.1:8000".to_owned());

    // Root credentials for administrative operations
    let root_user = env::var("SURREAL_ROOT_USER").unwrap_or_else(|_| "".to_owned());
    let root_pass = env::var("SURREAL_ROOT_PASS").unwrap_or_else(|_| "".to_owned());

    // Namespace credentials for namespace management
    let namespace_user = env::var("SURREAL_NAMESPACE_USER").ok().filter(|s| !s.is_empty());
    let namespace_pass = env::var("SURREAL_NAMESPACE_PASS").ok();

    // Database credentials for application operations
    let database_user = env::var("SURREAL_USERNAME")
        .or_else(|_| env::var("SURREAL_USER"))
        .ok()
        .filter(|s| !s.is_empty());
    let database_pass = env::var("SURREAL_PASSWORD")
        .or_else(|_| env::var("SURREAL_PASS"))
        .ok();

    let ns = env::var("SURREAL_NS").unwrap_or_else(|_| "rustblog".to_owned());
    let db_name = env::var("SURREAL_DB").unwrap_or_else(|_| "rustblog".to_owned());

    // Connect with enhanced error handling
    let client = Surreal::new::<&str>(&format!("{}://{}", protocol, host)).await?;

    // Authenticate with appropriate credentials
    if !root_user.is_empty() && !root_pass.is_empty() {
        client.signin(Root {
            username: &root_user,
            password: &root_pass,
        }).await?;
    }

    client.use_ns(ns).use_db(db_name).await?;

    // Use namespace/database credentials if available
    if let (Some(user), Some(pass)) = (database_user, database_pass) {
        client.signin(Namespace {
            username: &user,
            password: &pass,
        }).await?;
    }

    Ok(client)
}
```

#### Database Permissions
```surql
-- migrations/0001_initial_schema.surql
-- Define permissions for anonymous users (read-only access)
DEFINE ACCESS anonymous ON DATABASE TYPE RECORD
    SIGNUP NONE
    SIGNIN NONE
    FOR SELECT ON post, author;

-- Define permissions for authenticated users
DEFINE ACCESS user ON DATABASE TYPE RECORD
    SIGNUP NONE
    SIGNIN (
        ALLOW users TO signin WITH email, password
    )
    FOR SELECT, UPDATE ON post WHERE author = $auth.id
    FOR SELECT ON author;

-- Define permissions for administrators
DEFINE ACCESS admin ON DATABASE TYPE RECORD
    SIGNIN (
        ALLOW admins TO signin WITH email, password
    )
    FOR ALL ON post, author, activity;
```

### 2. Application Security

#### Input Validation
```rust
// app/src/api.rs - Secure input handling
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 200), custom = "validate_title")]
    pub title: String,

    #[validate(length(min = 1, max = 1000), custom = "validate_slug")]
    pub slug: String,

    #[validate(length(min = 1, max = 50000))]
    pub content: String,

    #[validate(length(max = 500))]
    pub excerpt: Option<String>,

    #[validate(length(max = 10))]
    pub tags: Option<Vec<String>>,
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    // Prevent XSS in title
    if title.contains('<') || title.contains('>') {
        return Err(ValidationError::new("invalid_html"));
    }
    Ok(())
}

fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    // Only allow alphanumeric, hyphens, and underscores
    if !slug.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ValidationError::new("invalid_slug"));
    }
    Ok(())
}

// Server function with validation
#[server(CreatePostRequest, "/api", "HttpPost")]
pub async fn create_post(
    req: CreatePostRequest,
) -> Result<Post, ServerFnError> {
    // Validate input
    if let Err(validation_errors) = req.validate() {
        return Err(ServerFnError::new("Validation failed"));
    }

    // Sanitize content (remove dangerous HTML)
    let sanitized_content = sanitize_html(&req.content);

    // Parameterized query to prevent SQL injection
    let query = "CREATE post SET title = $title, slug = $slug, content = $content, excerpt = $excerpt, published_at = time::now()";

    let mut result = DB.query(query)
        .bind(("title", req.title))
        .bind(("slug", req.slug))
        .bind(("content", sanitized_content))
        .bind(("excerpt", req.excerpt))
        .await?;

    // ... rest of implementation
}
```

#### Security Headers Middleware
```rust
// server/src/middleware/security.rs
use axum::{
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use tower_http::set_header::SetResponseHeaderLayer;

pub fn security_middleware() -> Vec<SetResponseHeaderLayer<HeaderValue, HeaderValue>> {
    vec![
        // Content Security Policy
        SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                 font-src 'self' https://fonts.gstatic.com; \
                 img-src 'self' data: https:; \
                 connect-src 'self'"
            ),
        ),

        // X-Frame-Options
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY")
        ),

        // X-Content-Type-Options
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff")
        ),

        // Referrer Policy
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin")
        ),

        // Strict-Transport-Security
        SetResponseHeaderLayer::overriding(
            axum::http::header::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains")
        ),
    ]
}
```

#### Rate Limiting
```rust
// server/src/middleware/rate_limit.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower::limit::RateLimitLayer;

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, client_ip: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let client_requests = requests.entry(client_ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        client_requests.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if under limit
        if client_requests.len() < self.max_requests as usize {
            client_requests.push(now);
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    rate_limiter: axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .split(',')
        .next()
        .unwrap_or("unknown")
        .trim();

    if !rate_limiter.check_rate_limit(client_ip) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
```

### 3. Infrastructure Security

#### Container Security
```dockerfile
# Dockerfile (optimized for security)
FROM rust:1.75-slim as builder

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY app/Cargo.toml ./app/
COPY server/Cargo.toml ./server/
COPY frontend/Cargo.toml ./frontend/
COPY markdown/Cargo.toml ./markdown/

# Create empty source files to prevent build errors
RUN mkdir -p app/src server/src frontend/src markdown/src && \
    touch app/src/lib.rs server/src/main.rs frontend/src/lib.rs markdown/src/lib.rs

# Download dependencies
RUN cargo build --workspace --release

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --workspace --release

# Production stage
FROM debian:bookworm-slim

# Install only necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy built application
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/target/release/leptos_server /app/
COPY --from=builder /app/target/site /app/site

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3007/health || exit 1

# Expose port
EXPOSE 3007

# Run the application
CMD ["./server"]
```

#### Environment Variable Validation
```rust
// server/src/config.rs
use std::env;

pub struct Config {
    pub database_url: String,
    pub database_user: String,
    pub database_pass: String,
    pub secret_key: String,
    // ... other config fields
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Validate required environment variables
        let database_url = env::var("SURREAL_ADDRESS")
            .map_err(|_| ConfigError::Missing("SURREAL_ADDRESS".to_string()))?;

        let database_user = env::var("SURREAL_USER")
            .map_err(|_| ConfigError::Missing("SURREAL_USER".to_string()))?;

        let database_pass = env::var("SURREAL_PASS")
            .map_err(|_| ConfigError::Missing("SURREAL_PASS".to_string()))?;

        let secret_key = env::var("SECRET_KEY")
            .map_err(|_| ConfigError::Missing("SECRET_KEY".to_string()))?;

        // Validate secret key length
        if secret_key.len() < 32 {
            return Err(ConfigError::Invalid("SECRET_KEY must be at least 32 characters".to_string()));
        }

        // Validate database URL format
        if !database_url.starts_with("http://") && !database_url.starts_with("https://") {
            return Err(ConfigError::Invalid("SURREAL_ADDRESS must be a valid URL".to_string()));
        }

        Ok(Config {
            database_url,
            database_user,
            database_pass,
            secret_key,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Invalid(String),
}
```

## 🔍 Security Scanning Implementation

### Multi-Tool Security Pipeline

The project implements security scanning using three complementary tools:

#### 1. Gitleaks - Secret Detection
```yaml
# .github/workflows/secrets-scan.yml
- name: Run Gitleaks
  uses: gitleaks/gitleaks-action@v2
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    GITLEAKS_LICENSE: ${{ secrets.GITLEAKS_LICENSE }}
  with:
    config: .gitleaks.toml
    fail: true
    verbose: true
```

Configuration:
```toml
# .gitleaks.toml
title = "Gitleaks Configuration"

[[rules]]
description = "GitHub Personal Access Token"
id = "github-pat"
regex = '''ghp_[a-zA-Z0-9]{36}'''
keywords = ["ghp_"]

[[rules]]
description = "AWS Access Key"
id = "aws-access-key"
regex = '''AKIA[0-9A-Z]{16}'''
keywords = ["AKIA"]

[[rules]]
description = "SurrealDB Token"
id = "surrealdb-token"
regex = '''surreal[a-zA-Z0-9]{32}'''
keywords = ["surreal"]
```

#### 2. Semgrep - Static Analysis
```yaml
# .github/workflows/secrets-scan.yml
- name: Run Semgrep
  uses: semgrep/semgrep-action@v1
  with:
    config: >-
      p/security-audit
      p/rust
      p/cwe-top-25
      .semgrep.yml
```

Custom rules:
```yaml
# .semgrep.yml
rules:
  - id: rust-sql-injection
    pattern: |
      db.query($QUERY + $INPUT)
    message: "Potential SQL injection - use parameterized queries instead"
    languages: [rust]
    severity: ERROR
    metadata:
      cwe: "CWE-89"

  - id: rust-hardcoded-credentials
    pattern: |
      $CREDENTIAL = "..."
    message: "Hardcoded credentials detected - use environment variables"
    languages: [rust]
    severity: ERROR
    metadata:
      cwe: "CWE-798"
```

#### 3. TruffleHog - Entropy-Based Detection
```yaml
# .github/workflows/secrets-scan.yml
- name: Run TruffleHog
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    base: main
    head: HEAD
    extra_args: --regex --entropy=False
```

### Automated Security Gates

```yaml
# .github/workflows/security-gate.yml
name: Security Gate

on:
  pull_request:
    branches: [ main ]
  push:
    branches: [ main ]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0  # Full history for scanning

    # Run all security tools
    - name: Gitleaks Secret Scanning
      uses: gitleaks/gitleaks-action@v2
      with:
        fail: true  # Block on findings

    - name: Semgrep Static Analysis
      uses: semgrep/semgrep-action@v1
      with:
        fail: true  # Block on findings

    - name: TruffleHog Entropy Scanning
      uses: trufflesecurity/trufflehog@main
      with:
        fail: true  # Block on findings

    # Dependency vulnerability scanning
    - name: Cargo Audit
      run: |
        cargo install cargo-audit
        cargo audit --deny warnings

    # Security report generation
    - name: Generate Security Report
      run: |
        ./scripts/generate-security-report.sh
      if: always()

    - name: Upload Security Artifacts
      uses: actions/upload-artifact@v4
      if: always()
      with:
        name: security-report-${{ github.run_number }}
        path: security-reports/
        retention-days: 90
```

## 🚨 Incident Response

### Security Incident Process

1. **Detection**
   - Automated scanning identifies vulnerability
   - Manual security review finding
   - External security disclosure

2. **Assessment**
   - Severity evaluation (Critical/High/Medium/Low)
   - Impact analysis
   - Affected systems identification

3. **Containment**
   - Immediate vulnerability fix
   - Temporary mitigations
   - System hardening

4. **Remediation**
   - Permanent fix implementation
   - Security regression testing
   - Documentation updates

5. **Reporting**
   - Security advisory publication
   - Patch release coordination
   - Stakeholder notification

### Emergency Response Contacts

- **Security Lead**: security@alexthola.com
- **GitHub Security**: security@github.com
- **Dependency Disclosures**: Responsible disclosure to maintainers

## 📊 Security Monitoring

### Continuous Security Monitoring

```bash
# scripts/monitor-security.sh
#!/bin/bash

# Daily security scan
echo "Running daily security scan..."

# Update vulnerability database
cargo audit --fetch

# Run security audit
cargo audit --deny warnings > security-audit-$(date +%Y%m%d).txt

# Check for new vulnerabilities
if cargo audit --deny warnings; then
    echo "✅ No new vulnerabilities detected"
else
    echo "🚨 New vulnerabilities detected!"
    # Send alert notification
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"🚨 Security alert: New vulnerabilities detected in blog engine"}' \
        $SLACK_WEBHOOK_URL
fi

# Run scan
./run_secret_scan.sh > secret-scan-$(date +%Y%m%d).txt

echo "Security scan completed"
```

### Security Metrics

- **Vulnerability Count**: Track open/closed security issues
- **Time to Remediation**: Measure response time for security findings
- **Scan Coverage**: Percentage of codebase covered by security tools
- **False Positive Rate**: Monitor security tool accuracy

## 🛠️ Best Practices

### Development Security

1. **Secure Coding Practices**
   ```rust
   // ✅ Good: Parameterized queries
   let result = db.query("SELECT * FROM posts WHERE id = $id")
       .bind(("id", post_id))
       .await?;

   // ❌ Bad: String concatenation
   let query = format!("SELECT * FROM posts WHERE id = {}", post_id);
   let result = db.query(&query).await?;
   ```

2. **Environment Variable Management**
   ```bash
   # ✅ Good: Use .env.example as template
   cp .env.example .env
   # Edit .env with actual values

   # ❌ Bad: Hardcode credentials
   export DATABASE_PASSWORD="supersecret123"
   ```

3. **Input Validation**
   ```rust
   // ✅ Good: validation
   #[derive(Validate)]
   struct CreateUserRequest {
       #[validate(email)]
       email: String,
       #[validate(length(min = 8))]
       password: String,
   }

   // ❌ Bad: No validation
   struct CreateUserRequest {
       email: String,
       password: String,
   }
   ```

### Operational Security

1. **Regular Updates**
   - Weekly dependency updates
   - Monthly security patching
   - Quarterly security reviews

2. **Access Control**
   - Least privilege principle
   - Regular access reviews
   - Multi-factor authentication

3. **Backup Security**
   - Encrypted backups
   - Regular restore testing
   - Off-site storage

---

**Related Documents**:
- [Architecture Overview](Architecture.md)
- [Database Guide](Database-Guide.md)
- [Troubleshooting](Troubleshooting.md)
- [Contributing Guide](Contributing-Guide.md)