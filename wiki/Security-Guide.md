# Security Guide

This document provides a detailed overview of the security measures implemented in the blog engine.

## Reporting a Vulnerability

Report any security vulnerabilities by emailing details to [alexthola@gmail.com](mailto:alexthola@gmail.com). **Do not create a public GitHub issue.**

Refer to the main [Security Policy](../SECURITY.md) for more information.

## Application Security

-   **Content Security Policy (CSP)**: A strict CSP is enforced via Axum middleware (defined in `server/src/security.rs`). This mitigates XSS risks by restricting which resources can be loaded.
-   **Rate Limiting**: An in-memory rate limiter is used to block IP addresses that make more than 100 requests per minute, preventing simple denial-of-service attacks.
-   **XSS Protection**: Leptos provides built-in XSS protection by automatically escaping all rendered data.
-   **SQL Injection Prevention**: All database queries are parameterized to prevent SQL injection attacks.

## Database Security

-   **Network Isolation**: The database runs on a separate Droplet and is only accessible from the application server over a private VPC network. A `ufw` firewall blocks all other external traffic.
-   **Strong Authentication**: The production database is secured with a 32-character randomly generated password.
-   **Automated Backups**: Daily backups of the database are automatically created with a 30-day retention policy.

## Secrets Management

-   **No Secrets in Git**: The repository does not contain any secrets. For local development, secrets are stored in a `.env` file, which is included in `.gitignore`. In production, they are managed as encrypted environment variables in the DigitalOcean App Platform.
-   **Automated Scanning**: Every commit is scanned for secrets by Gitleaks, Semgrep, and TruffleHog as part of the CI pipeline. A discovered secret will fail the build and block any deployment.

## Dependency Security

-   **Vulnerability Scanning**: Dependencies are scanned for known vulnerabilities weekly using `cargo audit` and Dependabot.
-   **Alerting**: Critical CVEs trigger immediate alerts, ensuring prompt attention.
-   **Patching Policy**: Critical vulnerabilities are patched within 24 hours of being identified; high-severity vulnerabilities are addressed within one week.

## CI/CD Security

The CI/CD pipeline acts as a security gate:
1.  **Secret Scan First**: The first step of every pipeline run is a comprehensive secret scan. A failure here stops the process immediately.
2.  **Build and Test Gate**: Code is only built and tested if the secret scan passes.
3.  **Conditional Deployment**: The application is only deployed to production if all prior security scans, tests, and checks pass on the `master` branch.

The `master` branch is protected, requiring pull request reviews and passing CI checks before merging.

## Infrastructure Security

-   **Managed Platform**: The application is deployed on the DigitalOcean App Platform, which handles OS patching, DDoS protection, and underlying network security.
-   **Firewall Management**: The primary infrastructure responsibility is maintaining the database Droplet's firewall. It is configured to block all traffic except for SSH and database connections from the app server's private VPC IP.

## Security Checklist

This checklist is verified before every deployment.

-   [x] SSL/TLS is active and enforces HTTPS.
-   [x] Strong, unique passwords are used for the database.
-   [x] The database is only accessible from the app's private IP address.
-   [x] All secrets are stored as encrypted environment variables.
-   [x] Rate limiting is enabled on public endpoints.
-   [x] Automated database backups are configured and functional.
-   [x] Secret scanning passes on the latest commit.
-   [x] Dependencies have been audited for new CVEs.