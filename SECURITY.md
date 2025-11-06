# Security Policy

## Overview

This document outlines the security measures, policies, and best practices implemented in the alexthola.com blog application.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **DO NOT** open a public GitHub issue
2. Email security concerns to: alexthola@gmail.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

**Response time**: We aim to respond within 48 hours and provide a fix within 7 days for critical vulnerabilities.

## Security Measures

### 1. Application Security

#### Security Headers

The application implements comprehensive security headers via middleware (server/src/security.rs:security_headers):

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Frame-Options` | `DENY` | Prevent clickjacking attacks |
| `X-Content-Type-Options` | `nosniff` | Prevent MIME type sniffing |
| `X-XSS-Protection` | `1; mode=block` | Enable browser XSS protection |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer information leakage |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Force HTTPS for 1 year |
| `Content-Security-Policy` | (see below) | Restrict resource loading |
| `Permissions-Policy` | (restrictive) | Disable unnecessary browser features |

**Content Security Policy (CSP):**
```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';
style-src 'self' 'unsafe-inline';
img-src 'self' data: https:;
font-src 'self' data:;
connect-src 'self';
frame-ancestors 'none';
base-uri 'self';
form-action 'self';
```

#### Rate Limiting

- **Implementation**: In-memory rate limiter (server/src/security.rs:RateLimiter)
- **Limit**: 100 requests per minute per IP address
- **Response**: HTTP 429 (Too Many Requests) when exceeded
- **Tracking**: By client IP address (from X-Forwarded-For header or direct connection)

#### HTTPS Enforcement

- Automatic HTTPS redirect via DigitalOcean App Platform
- HSTS header enforces HTTPS for 1 year
- Let's Encrypt SSL/TLS certificates (auto-renewed)
- Minimum TLS version: 1.2

#### Input Validation

- Server-side validation for all user inputs
- Leptos framework provides built-in XSS protection
- Database queries use parameterized statements (via SurrealDB)
- No direct HTML rendering of user content

### 2. Database Security

#### SurrealDB Configuration

- **Authentication**: Three-tier authentication system
  1. Database-level (SURREAL_USERNAME/PASSWORD) - Recommended
  2. Namespace-level (SURREAL_NAMESPACE_USER/PASS)
  3. Root-level (SURREAL_ROOT_USER/PASS)

- **Network Security**:
  - Database listens on private network only
  - Firewall restricts access to application VPC
  - No public database exposure

- **Password Requirements**:
  - Production: Minimum 32 characters
  - Development: Minimum 8 characters
  - Generated using: `openssl rand -base64 32`

- **Connection Security**:
  - Automatic retry with exponential backoff
  - Connection pooling (via SurrealDB client)
  - Graceful degradation on connection failure

#### Data Protection

- **Backups**: Automated daily backups (cron job)
- **Retention**: 30 days of backups
- **Encryption**: At-rest encryption on DigitalOcean Droplet
- **Access Control**: Role-based access control (SurrealDB)

### 3. Secrets Management

#### Environment Variables

All secrets are managed via environment variables, never hardcoded:

**Development** (.env):
- Local development only
- Never committed to git (.gitignore)
- Uses weak credentials for convenience

**Production** (DigitalOcean):
- Encrypted environment variables in App Platform
- Injected at runtime
- Never logged or exposed

**Required Secrets**:
- `SURREAL_PASSWORD` - Database password (encrypted)
- `SURREAL_ADDRESS` - Database connection URL
- `DIGITALOCEAN_ACCESS_TOKEN` - CI/CD deployment (GitHub Secrets)

#### Secret Scanning

Automated secret scanning on every commit:

1. **Gitleaks**: Detects hardcoded secrets in files and git history
2. **Semgrep**: Identifies dangerous code patterns
3. **TruffleHog**: Entropy analysis for credential-like strings

**False positives**: Add to `.gitleaksignore` with justification

### 4. Dependency Security

#### Automated Audits

- **cargo audit**: Checks for known vulnerabilities (Rust Security Advisory Database)
- **GitHub Dependabot**: Automated dependency updates
- **Frequency**: Weekly scans, immediate alerts for critical CVEs

#### Update Policy

- **Critical vulnerabilities**: Patched within 24 hours
- **High severity**: Patched within 1 week
- **Medium severity**: Patched within 1 month
- **Low severity**: Patched in next scheduled maintenance

#### Minimal Dependencies

- Only essential dependencies included
- Regular review and removal of unused dependencies
- Prefer well-maintained, audited crates

### 5. CI/CD Security

#### GitHub Actions

**Security Workflow Order**:
```
1. secrets-scan.yml (Security Gate)
   └─ Blocks deployment if secrets detected
2. rust.yml (Build & Test)
   └─ Runs only if secrets-scan passes
3. migrations.yml (Database)
   └─ Validates migrations
4. deploy.yml (Production)
   └─ Deploys only if all previous workflows pass
```

**Branch Protection**:
- Master branch requires PR reviews
- All CI checks must pass before merge
- No direct commits to master

**Secrets Storage**:
- GitHub Secrets (encrypted at rest)
- Scoped to specific workflows
- Never logged or exposed in output

### 6. Infrastructure Security

#### DigitalOcean App Platform

- **Automatic Updates**: Platform security patches auto-applied
- **DDoS Protection**: Built-in Layer 3/4 DDoS mitigation
- **Network Isolation**: VPC network isolation
- **Monitoring**: Real-time security monitoring

#### Firewall Configuration

**Application** (App Platform):
- Only ports 80/443 exposed (HTTP/HTTPS)
- Automatic redirect 80 → 443

**Database** (Droplet):
- UFW firewall enabled
- SSH: Port 22 (restricted to admin IPs)
- SurrealDB: Port 8000 (VPC only)
- All other ports: Denied

**Firewall Rules**:
```bash
ufw allow 22/tcp    # SSH (restrict to known IPs in production)
ufw allow from VPC_RANGE to any port 8000  # Database (VPC only)
ufw deny incoming   # Deny all other inbound
ufw allow outgoing  # Allow outbound
```

### 7. Monitoring and Incident Response

#### Security Monitoring

- **Application Logs**: Centralized logging (DigitalOcean)
- **Anomaly Detection**: Rate limit violations logged
- **Alerting**: Email alerts for security events

**Monitored Events**:
- Failed authentication attempts
- Rate limit violations (429 responses)
- Unexpected errors (500 responses)
- Database connection failures

#### Incident Response Plan

1. **Detection**: Automated alerts or manual discovery
2. **Assessment**: Determine scope and impact (< 1 hour)
3. **Containment**: Isolate affected systems (< 2 hours)
4. **Eradication**: Remove threat and patch vulnerability (< 24 hours)
5. **Recovery**: Restore services and verify integrity (< 48 hours)
6. **Post-Mortem**: Document incident and improve security

### 8. Security Best Practices

#### For Developers

- [ ] Never commit secrets to git
- [ ] Run `./run_secret_scan.sh` before every commit
- [ ] Use strong, unique passwords (min 32 chars for production)
- [ ] Keep dependencies up to date (`cargo update` weekly)
- [ ] Review security headers with `curl -I https://alexthola.com`
- [ ] Test rate limiting manually before deployment
- [ ] Validate environment variables at startup

#### For Operators

- [ ] Rotate credentials every 90 days
- [ ] Review access logs weekly
- [ ] Monitor security alerts daily
- [ ] Test backups monthly
- [ ] Update firewall rules when infrastructure changes
- [ ] Enable 2FA on all admin accounts (GitHub, DigitalOcean, NameCheap)
- [ ] Restrict database access to VPC only

#### For Users

- Blog is read-only for visitors (no user authentication required)
- All data transmitted over HTTPS
- No personal data collected (privacy-first approach)
- Optional analytics can be privacy-focused (Plausible, Fathom)

## Security Checklist for Production Deployment

Before deploying to production, verify:

- [x] SSL/TLS certificate active and valid
- [x] HTTPS enforced (HTTP redirects to HTTPS)
- [x] Security headers present (X-Frame-Options, CSP, HSTS, etc.)
- [x] Database uses strong, unique password (min 32 characters)
- [x] Database firewall-protected (VPC only)
- [x] All secrets stored as encrypted environment variables
- [x] Rate limiting enabled (100 req/min per IP)
- [x] Automated backups configured
- [x] Monitoring alerts set up
- [x] Dependencies up to date (no known CVEs)
- [x] Secret scanning passes (Gitleaks, Semgrep, TruffleHog)
- [x] www subdomain redirects to apex domain
- [x] Health endpoint returns 200 OK
- [x] Application logs don't expose secrets
- [x] Input validation implemented
- [x] Error messages don't leak sensitive info

## Compliance

### Data Privacy

- **No user accounts**: No personal data stored
- **No cookies**: Except session cookies for functionality
- **No tracking**: Optional privacy-focused analytics only
- **GDPR compliant**: No personal data processing

### Logging

- **What we log**:
  - Request paths and response codes
  - IP addresses (for rate limiting)
  - Timestamps
  - Error messages

- **What we DON'T log**:
  - Passwords or credentials
  - Personal data
  - Request bodies (except for debugging)

- **Retention**: 7 days (DigitalOcean default)

## Security Testing

### Automated Testing

- **Unit tests**: Security middleware (server/src/security.rs)
- **Integration tests**: Full request/response flow
- **Secret scanning**: Every commit (GitHub Actions)
- **Dependency audit**: Weekly (cargo audit)

### Manual Testing

- **Security headers**: `curl -I https://alexthola.com`
- **Rate limiting**: Load testing with `ab` or `wrk`
- **SSL/TLS**: https://www.ssllabs.com/ssltest/
- **Security headers**: https://securityheaders.com/
- **Penetration testing**: Recommended annually

## Updates and Maintenance

### Security Updates

- **Critical**: Immediate deployment (within 24 hours)
- **High**: Scheduled deployment (within 1 week)
- **Medium/Low**: Regular maintenance window

### Maintenance Schedule

| Task | Frequency |
|------|-----------|
| Security audit | Monthly |
| Dependency updates | Weekly |
| Credential rotation | Quarterly |
| Backup testing | Monthly |
| Firewall review | Quarterly |
| Penetration testing | Annually |

## Contact

For security concerns:
- Email: alexthola@gmail.com
- GitHub: https://github.com/athola/blog/issues (non-sensitive issues only)

## Changelog

- **2025-11-05**: Initial security policy
  - Added security headers middleware
  - Implemented rate limiting
  - Added environment validation
  - Documented all security measures

---

**Last Updated**: 2025-11-05
**Version**: 1.0.0
