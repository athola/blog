# Security Guide

This document details the security measures for the blog engine.

## Application Security

### Security Headers
A strict Content Security Policy (CSP) and other security headers are set via Axum middleware in `server/src/security.rs`. This helps prevent attacks like clickjacking and XSS by restricting resource loading to trusted domains.

### Rate Limiting
An in-memory rate limiter is configured to prevent simple denial-of-service attacks. It blocks IP addresses that make more than 100 requests per minute.

### Input Validation and XSS
The application uses Leptos's built-in XSS protection, which automatically escapes all rendered data. All database queries are parameterized to prevent SQL injection.

## Database Security

-   **Network Isolation**: The database runs on a separate server and is only accessible over a private VPC network. A firewall blocks all traffic except from the application server.
-   **Authentication**: The production database is protected by a 32-character generated password.
-   **Backups**: Daily backups are automatically created with a 30-day retention policy.

## Secrets Management

-   **No Secrets in Git**: Secrets are never committed to the repository. They are managed using a `.gitignore`'d `.env` file for local development and encrypted environment variables in production.
-   **Automated Scanning**: Every commit is scanned for secrets by Gitleaks, Semgrep, and TruffleHog in a GitHub Actions workflow. A detected secret will fail the build and block deployment.

## Dependency Security
Dependencies are scanned for vulnerabilities weekly using `cargo audit` and Dependabot.
-   **Alerts**: Critical CVEs trigger immediate alerts.
-   **Patching Policy**: Critical vulnerabilities are patched within 24 hours; high-severity ones within one week.

## CI/CD Security
The GitHub Actions pipeline is designed to prevent security issues from being deployed.
1.  **Secret Scan**: The first step of every build is a secret scan. A failure here blocks deployment.
2.  **Build and Test**: The code is only built if the secret scan passes.
3.  **Deploy**: The application is only deployed if all tests and checks pass.

The `master` branch is protected, requiring pull request reviews and passing CI checks before merging.

## Infrastructure Security
The application is deployed on the DigitalOcean App Platform, which manages OS patching, DDoS protection, and network security.

The main infrastructure responsibility for this project is the database firewall, which is configured to block all traffic except for SSH and database connections from the application server.

## Pre-Deployment Security Checklist

-   [x] SSL/TLS is active and enforces HTTPS.
-   [x] Strong, unique passwords are in place for the database.
-   [x] The database is only accessible from the app's private IP.
-   [x] All secrets are stored as encrypted environment variables.
-   [x] Rate limiting is enabled.
-   [x] Automated backups are configured and working.
-   [x] Secret scanning is passing for the latest commit.
-   [x] Dependencies have been audited for CVEs.