# Security Guide

This document provides a detailed overview of the security measures, policies, and best practices for this application. The security approach is based on defense-in-depth, combining automated scanning, secure configuration, and regular maintenance.

## Security Measures

### 1. Application Security

The application is configured to be secure by default.

#### Security Headers

A strong set of security headers are implemented via Axum middleware (`server/src/security.rs`). These headers protect against common web vulnerabilities like clickjacking, XSS, and protocol downgrades. The Content Security Policy (CSP) is configured to restrict resource loading to trusted sources, preventing many types of injection attacks.

#### Rate Limiting

To mitigate simple denial-of-service attacks, an in-memory rate limiter is in place. It tracks requests by IP address and returns an HTTP 429 response if a client exceeds 100 requests per minute.

#### Input Validation & XSS

The application relies on the Leptos framework's built-in protections against Cross-Site Scripting (XSS). All user input is treated as data, not executable HTML. Database queries use SurrealDB's parameterized statements to prevent SQL injection.

### 2. Database Security

The database is isolated and hardened.

-   **Network Isolation:** The SurrealDB instance runs on a separate server and listens only on a private VPC network. A firewall restricts traffic to the application server's IP address, and the database is not exposed to the public internet.
-   **Authentication:** The production database uses a strong, 32-character randomly generated password.
-   **Backups:** Automated daily backups are configured with a 30-day retention period.

### 3. Secrets Management

Secrets are never committed to the git repository.

-   **Local Development:** A `.env` file, which is excluded from git via `.gitignore`, is used for local development.
-   **Production:** Production secrets are managed as encrypted environment variables in the DigitalOcean App Platform. They are injected at runtime and are not recorded in logs.
-   **Automated Scanning:** Every commit triggers a GitHub Actions workflow that runs three secret scanners: Gitleaks, Semgrep, and TruffleHog. If a secret is detected, the build fails, and deployment is blocked.

### 4. Dependency Security

Dependencies are monitored for known vulnerabilities using `cargo audit` and GitHub's Dependabot.
-   **Frequency:** Scans are performed weekly.
-   **Alerts:** Immediate alerts are generated for critical CVEs.
-   **Patching Policy:** Critical vulnerabilities are patched within 24 hours, and high-severity vulnerabilities are patched within one week.

### 5. CI/CD Security

The GitHub Actions pipeline is designed with a security-first approach.

1.  **Secret Scan Gate:** The first step in any workflow is a secret scan. A failure at this stage stops the entire run.
2.  **Build & Test:** Code is only built if the security scan passes.
3.  **Deployment:** The application is only deployed if all preceding steps, including tests, are successful.

Branch protection rules for the `master` branch require pull request reviews and passing CI checks before merging.

### 6. Infrastructure Security

The DigitalOcean App Platform provides managed infrastructure security, including OS patching, DDoS protection, and network isolation.

The primary infrastructure security responsibility for this project is the database server's firewall. It is configured with `ufw` to deny all incoming traffic except for SSH (from a trusted IP) and SurrealDB traffic (from the application's private IP).

## Pre-Deployment Security Checklist

-   [x] SSL/TLS is active and enforces HTTPS.
-   [x] Strong, unique passwords are in place for the database.
-   [x] The database is only accessible from the app's private IP.
-   [x] All secrets are stored as encrypted environment variables.
-   [x] Rate limiting is enabled.
-   [x] Automated backups are configured and working.
-   [x] Secret scanning is passing for the latest commit.
-   [x] Dependencies have been audited for CVEs.