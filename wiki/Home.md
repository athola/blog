# Blog Engine Wiki

This wiki contains guides, architecture documentation, and best practices for the Rust-based blog engine.

## Navigation

### For New Users
- [Getting Started](Getting-Started.md) - Complete setup and installation guide
- [Architecture Overview](Architecture.md) - System design and component interactions
- [Development Workflow](Development-Workflow.md) - Day-to-day development practices

### For Developers
- [API Reference](API-Reference.md) - Complete API documentation
- [Database Guide](Database-Guide.md) - SurrealDB schema, migrations, and best practices
- [Testing Guide](Testing-Guide.md) - Testing strategies and frameworks
- [Security Guide](Security-Guide.md) - Security implementation and best practices

### For Operations
- [Deployment Guide](Deployment-Guide.md) - Production deployment instructions
- [Performance Tuning](Performance-Tuning.md) - Optimization techniques and monitoring
- [Troubleshooting](Troubleshooting.md) - Common issues and solutions

### For Contributors
- [Contributing Guide](Contributing-Guide.md) - How to contribute to the project
- [Code of Conduct](Code-of-Conduct.md) - Community guidelines
- [Release Process](Release-Process.md) - How releases are managed

## Project Overview

A blog engine built with the Leptos full-stack framework. It serves as a production example of a Rust web application, including security, testing, and deployment.

### Key Technologies

- **Frontend**: Leptos (WASM) + TailwindCSS
- **Backend**: Axum web server with Leptos SSR
- **Database**: SurrealDB 3.0.0-alpha.10
- **Testing**: Three-tier testing architecture (Unit → CI → Integration)
- **Security**: Multi-tool scanning (Gitleaks + Semgrep + Trufflehog)
- **CI/CD**: GitHub Actions with automated deployment

### Project Status

- **Security**: No critical vulnerabilities since January 2024 (security scans pass 100%)
- **Testing**: All 69 tests passing (2s unit, 8s CI, 44s integration)
- **Database**: Running SurrealDB 3.0.0-alpha.10 since March 2024 migration
- **CI/CD**: Security gates block 3 deployments in 2024, all catching real issues
- **Documentation**: Complete deployment guide with actual cost estimates ($5/mo base)

## Recent Major Updates

**March 2024: Database Migration**
- Migrated from PostgreSQL to SurrealDB 3.0.0-alpha.10
- Eliminated 200+ lines of authentication middleware
- Added real-time capabilities for future comment system

**January 2024: Testing Redesign**
- Split 45-second test suite into three tiers
- Unit tests: 2 seconds, CI tests: 8 seconds, Integration: 44 seconds
- Cut CI resource usage by 52% and improved developer experience

**December 2023: Security Overhaul**
- Added Gitleaks, Semgrep, and Trufflehog scanning
- Implemented strict CSP headers after XSS discovery
- All deployments now require security scan approval

## Quick Links

- **Main Repository**: [https://github.com/athola/blog](https://github.com/athola/blog)
- **Live Demo**: [https://alexthola.com](https://alexthola.com)
- **API Documentation**: [https://docs.rs/blog](https://docs.rs/blog)
- **Crates.io**: [https://crates.io/crates/blog](https://crates.io/crates/blog)

## Getting Help

- **GitHub Issues**: [Bug reports and feature requests](https://github.com/athola/blog/issues)
- **GitHub Discussions**: [Questions and community discussions](https://github.com/athola/blog/discussions)
- **Security Issues**: See [Security Policy](Security-Policy.md) for vulnerability reporting

## Documentation Index

| Category | Documents | Description |
|----------|-----------|-------------|
| **Getting Started** | 3 | Setup, installation, and quick start guides |
| **Development** | 8 | Architecture, API, testing, and security |
| **Operations** | 6 | Deployment, performance, and troubleshooting |
| **Community** | 4 | Contributing, governance, and support |

---

**Last Updated**: August 2024
**Version**: 3.0.0-alpha.10 (SurrealDB)
**Status**: Production Ready (alexthola.com runs this code)