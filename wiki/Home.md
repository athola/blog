# Blog Engine Wiki

Welcome to the documentation for the Rust-based blog engine. This wiki provides detailed guides, architectural documentation, and best practices for developers and contributors.

## 🚀 Quick Navigation

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

## 🏗️ Project Overview

This is a blog engine built entirely in Rust using the Leptos full-stack framework. The project demonstrates production-ready Rust web development with security, testing, and deployment practices.

### Key Technologies

- **Frontend**: Leptos (WASM) + TailwindCSS
- **Backend**: Axum web server with Leptos SSR
- **Database**: SurrealDB 3.0.0-alpha.10
- **Testing**: Three-tier testing architecture (Unit → CI → Integration)
- **Security**: Multi-tool scanning (Gitleaks + Semgrep + Trufflehog)
- **CI/CD**: GitHub Actions with automated deployment

### Project Status

- ✅ **Security**: Secure with zero critical vulnerabilities
- ✅ **Testing**: 100% test pass rate (69/69 tests passing)
- ✅ **Database**: Modern SurrealDB 3.0.0-alpha.10 with enhanced authentication
- ✅ **CI/CD**: Automated deployment with security gates
- ✅ **Documentation**: Guides and API reference

## 📋 Recent Major Updates (December 2024)

We have recently completed a major overhaul of the database, testing, and security infrastructure. The database has been migrated to SurrealDB 3.0.0-alpha.10, with enhanced authentication and connection resilience. The testing architecture has been updated to a three-tier strategy, resulting in a 50% reduction in CI resource consumption. The security infrastructure has been enhanced with a multi-tool security scanning pipeline and automated security gates. The build system has been modernized with cargo-make integration and an improved development workflow.

## 🔗 Quick Links

- **Main Repository**: [https://github.com/athola/blog](https://github.com/athola/blog)
- **Live Demo**: [https://alexthola.com](https://alexthola.com)
- **API Documentation**: [https://docs.rs/blog](https://docs.rs/blog)
- **Crates.io**: [https://crates.io/crates/blog](https://crates.io/crates/blog)

## 📞 Getting Help

- **GitHub Issues**: [Bug reports and feature requests](https://github.com/athola/blog/issues)
- **GitHub Discussions**: [Questions and community discussions](https://github.com/athola/blog/discussions)
- **Security Issues**: See [Security Policy](Security-Policy.md) for vulnerability reporting

## 📖 Documentation Index

| Category | Documents | Description |
|----------|-----------|-------------|
| **Getting Started** | 3 | Setup, installation, and quick start guides |
| **Development** | 8 | Architecture, API, testing, and security |
| **Operations** | 6 | Deployment, performance, and troubleshooting |
| **Community** | 4 | Contributing, governance, and support |

---

**Last Updated**: December 2024
**Version**: 3.0.0-alpha.10 (SurrealDB)
**Status**: Production Ready ✅