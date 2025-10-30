# Blog

[![Crates.io](https://img.shields.io/crates/v/blog.svg)](https://crates.io/crates/blog)
[![Documentation](https://docs.rs/blog/badge.svg)](https://docs.rs/blog)
[![License: 0BSD](https://img.shields.io/badge/License-0BSD-blue.svg)](https://opensource.org/licenses/0BSD)
[![Build Status](https://github.com/athola/blog/workflows/CI/badge.svg)](https://github.com/athola/blog/actions)
[![Coverage](https://img.shields.io/codecov/c/github/athola/blog)](https://codecov.io/gh/athola/blog)

A modern, fast, and secure blog engine built with Rust using the Leptos full-stack framework.

## Overview

Blog is a Rust-powered content management system that combines the performance and safety of Rust with modern web development practices. Built on the Leptos framework, it provides server-side rendering with client-side hydration for optimal performance and user experience.

### Core Features

This blog is a full-stack Rust application powered by the Leptos framework. It features server-side rendering with client-side hydration for fast page loads, and a responsive design using TailwindCSS. The backend uses a SurrealDB 3.0.0-alpha.10 database with advanced authentication and reliability features. The blog supports rich content with Markdown, syntax highlighting, and math support. It also includes a contact system with email integration and robust retry mechanisms. Security is a priority, with a multi-tool security scanning pipeline (Gitleaks + Semgrep + Trufflehog) and automated vulnerability detection. The project has a three-tier testing architecture (Unit → CI-optimized → Integration) with a 100% pass rate. CI/CD is automated with GitHub Actions, including security gates and automated deployment to DigitalOcean. Code quality is maintained with automated linting, formatting, PR size management, and validation.

### Technology Stack

- **Frontend**: Leptos (WASM) + TailwindCSS
- **Backend**: Axum web server with Leptos SSR
- **Database**: SurrealDB 3.0.0-alpha.10 (latest production-ready alpha)
- **Build System**: cargo-leptos + modernized Makefile with cargo-make integration
- **Testing**: nextest + cargo-llvm-cov for coverage analysis with CI-aware optimizations
- **Security**: Gitleaks, Semgrep, Trufflehog multi-tool scanning with automated gates
- **Development**: Enhanced database readiness scripts and improved connection handling

## Quick Start

### Prerequisites

- Rust (latest stable) with WASM target: `rustup target add wasm32-unknown-unknown`
- [SurrealDB 3.0.0-alpha.10](https://surrealdb.com/install) (required - not backwards compatible)
- Required cargo tools: `make install-pkgs` (installs cargo-leptos, cargo-nextest, and more)

### Installation

```bash
# Clone the repository
git clone https://github.com/athola/blog.git
cd blog

# Install required tools and dependencies
make install-pkgs

# Install SurrealDB if not already present
make install-surrealdb

# Copy environment configuration
cp .env.example .env
# Edit .env with your configuration

# Initialize database with setup
make init-db
```

### Development

```bash
# Start development server with live reload and database
make watch

# The application runs on http://127.0.0.1:3007
# Live reload available on http://127.0.0.1:3001

# Run tests
make test

# Code quality checks
make lint
make format

# Security scanning
./run_secret_scan.sh
```

### Production Build

```bash
# Build optimized release artifacts
make build-release

# Generate test coverage report
make test-coverage-html

# Run production server
make server-release
```

## Example Usage

### Creating a New Blog Post

```rust
use app::types::Post;
use serde_json::json;

// Create post via API
let post_data = json!({
    "title": "My First Blog Post",
    "slug": "my-first-post",
    "content": "# Hello World\n\nThis is my first post!",
    "excerpt": "An introduction to my new blog",
    "tags": ["rust", "web", "leptos"]
});

// POST to /api/posts with authentication
```

### Custom Styling

```rust
// app/src/components/custom.rs
use leptos::prelude::*;

#[component]
pub fn CustomHeader() -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-blue-600 to-purple-600 text-white">
            <nav class="container mx-auto px-6 py-4">
                <h1 class="text-2xl font-bold">"My Blog"</h1>
            </nav>
        </header>
    }
}
```

## Architecture

### Project Structure

```
blog/
├── app/                    # Shared application logic
│   ├── src/
│   │   ├── components/     # Leptos UI components
│   │   ├── api.rs         # Server function definitions
│   │   ├── types.rs       # Shared data types
│   │   └── lib.rs         # Main Leptos app
│   └── Cargo.toml         # Frontend dependencies
├── server/                 # Axum web server
│   ├── src/
│   │   ├── main.rs        # Server entry point
│   │   └── utils.rs       # Database and utility functions
│   └── Cargo.toml         # Server dependencies
├── frontend/              # WASM frontend entry point
│   └── Cargo.toml
├── markdown/              # Markdown processing utilities
├── migrations/            # Database schema definitions
├── tests/                 # Integration and performance tests
├── style/                 # TailwindCSS configuration
├── .github/workflows/     # CI/CD pipeline definitions
├── Makefile              # Build and development commands
└── README.md             # This file
```

### Data Flow

```
[Client] → HTTP Request → [Axum Server] → [Leptos SSR] → [SurrealDB]
                      ↓                ↓
              [Response HTML]  ←  [WASM Hydration] ← [Client-side Interactivity]
```

## Testing Strategy

The project uses a three-tier testing approach:

### Test Suites

```bash
# Full test suite (all tiers)
make test                           # 69/69 tests passing ✅

# Specialized test suites
make test-unit               # Unit tests only (~0s)
make test-ci                 # CI-optimized integration tests (~5s)
make test-server             # Full integration tests (~44s)
make test-db                 # Database-focused tests
make test-email              # Email functionality tests
make test-retry              # Retry mechanism tests
make test-migrations         # Migration validation tests
make test-server-integration # Standalone server integration tests

# Coverage analysis
make test-coverage-html      # Generate HTML coverage report

# Full validation
make validate                 # Full pipeline: format + lint + test + security
```

### Test Architecture

- **Unit Tests**: Fast, isolated component testing (~0s execution)
- **CI Tests**: Resource-conscious for pipeline efficiency (~5s execution, 50% resource reduction)
- **Integration Tests**: Full workflow validation with real database (~44s execution)
- **Performance Tests**: Load testing and benchmarking with CI-aware timeouts
- **Security Tests**: Multi-tool vulnerability scanning and penetration testing
- **Database Tests**: SurrealDB 3.0.0-alpha.10 compatibility validation
- **Resource Optimization**: Pattern-based test targeting with automatic new test inclusion

## Security

### Security Scanning

The project implements security monitoring:

- **Multi-tool Scanning**: Gitleaks + Semgrep + Trufflehog
- **Automated CI Gates**: Security findings block deployment
- **Dependency Auditing**: Cargo audit for known vulnerabilities
- **False Positive Management**: Fingerprint-based ignore system
- **Weekly Scans**: Ongoing security monitoring

```bash
# Run security scan
./run_secret_scan.sh

# Results saved to secret_scanning_results/
```

### Security Features

- Environment variable validation and SurrealDB 3.0.0-alpha.10 authentication
- Input sanitization and parameterized queries with SQL injection prevention
- Security headers (CSP, X-Frame-Options, etc.) with middleware
- Rate limiting for public endpoints and enhanced database connection security
- Non-root container execution and CI/CD workflow security hardening
- Secrets management with `.env.example` template and fingerprint-based false positive handling
- Zero critical vulnerabilities with automated security gates and weekly monitoring

## Documentation

### Essential Documentation

- [**Deployment Guide**](DEPLOYMENT.md) - Complete DigitalOcean deployment instructions
- [**Project Plan**](PLAN.md) - Detailed roadmap and technical specifications
- [**Contributing Guide**](CONTRIBUTING.md) - Development guidelines and PR process
- [**Security Policy**](SECURITY.md) - Vulnerability reporting and security practices
- [**API Documentation**](https://docs.rs/blog) - Rust API reference
- [**Examples**](examples/) - Code examples and tutorials

### Development Documentation

- [Architecture Overview](docs/architecture.md) - System design and patterns
- [Database Schema](docs/database.md) - SurrealDB schema and migrations
- [Testing Guide](docs/testing.md) - Testing strategies and best practices
- [Performance Guide](docs/performance.md) - Optimization techniques and benchmarks
- [Security Guide](docs/security.md) - Security implementation details

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Checklist

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Ensure** all tests pass: `make test`
4. **Run** code quality checks: `make lint && make format`
5. **Run** security scan: `./run_secret_scan.sh`
6. **Commit** with conventional commit format
7. **Push** to your fork and submit a Pull Request

### Development Workflow

```bash
# Start development environment
make watch                    # Starts database + live reload server

# Make changes with live reload
# Edit files in app/, server/, or frontend/

# Run test suite
make test                     # All tests must pass

# Quality checks
make lint                     # Clippy linting
make format                   # Rust formatting
make security                 # Security audit

# Validate before PR
make validate                 # Full validation pipeline
```

## Performance

### Benchmarks

- **Page Load**: < 2s initial, < 0.5s subsequent navigations
- **Database Queries**: < 100ms average response time
- **Bundle Size**: ~150KB gzipped WASM bundle
- **Test Coverage**: 95%+ line coverage
- **Uptime**: 99.9% availability target

### Optimization Features

- **HTTP Compression**: gzip, brotli, deflate, zstd
- **Asset Optimization**: Minified CSS/JS with content hashing
- **Database Connection Pooling**: Efficient connection management
- **Caching Strategy**: Multi-level caching for optimal performance
- **CDN Ready**: Static asset optimization for CDN deployment

## Deployment

### Production Deployment

The project includes automated deployment to DigitalOcean:

```bash
# Production deployment (main branch only)
git push origin main          # Triggers automated deployment

# Manual deployment verification
./scripts/smoke-tests.sh      # Post-deployment health checks
```

### Platform Support

- **DigitalOcean App Platform**: Automated deployment with CI/CD
- **Docker**: Containerized deployment with multi-stage builds
- **Self-hosted**: Manual deployment with provided scripts
- **Development**: Local development with hot reload

### Environment Configuration

```bash
# Required environment variables
export SURREAL_HOST="127.0.0.1:8000"
export SURREAL_NS="production"
export SURREAL_DB="blog"
export LEPTOS_SITE_ADDR="0.0.0.0:8080"
export RUST_LOG="info"
```

## Community

### Getting Help

- **Documentation**: Check the [docs/](docs/) directory for detailed guides
- **Issues**: [GitHub Issues](https://github.com/athola/blog/issues) for bug reports and feature requests
- **Discussions**: [GitHub Discussions](https://github.com/athola/blog/discussions) for questions and ideas
- **Security**: See [SECURITY.md](SECURITY.md) for vulnerability reporting

### Related Projects

- [Leptos Framework](https://leptos.dev/) - Full-stack Rust web framework
- [Axum Web Server](https://github.com/tokio-rs/axum) - Ergonomic and modular web framework
- [SurrealDB](https://surrealdb.com/) - Modern multi-model database
- [TailwindCSS](https://tailwindcss.com/) - Utility-first CSS framework

## Roadmap

### Current Development 🚧

- [x] **Security Infrastructure**: Multi-tool security scanning and CI gates (COMPLETED)
- [x] **Test Reliability**: Three-tier testing architecture with 100% pass rate (COMPLETED)
- [x] **Database Modernization**: SurrealDB 3.0.0-alpha.10 with enhanced authentication (COMPLETED)
- [x] **Build System Enhancement**: Modernized Makefile with cargo-make integration (COMPLETED)
- [ ] **Performance Optimization**: CI/CD pipeline optimization (46% faster target)
- [ ] **Enhanced Testing**: WASM-specific test configuration and SAST implementation
- [ ] **Security Hardening**: Advanced dependency scanning and code analysis

### Upcoming Features 📋

- [ ] **User Experience**: Dark mode, syntax highlighting, search functionality
- [ ] **Content Management**: Admin interface, draft workflow, content versioning
- [ ] **Community Features**: Comments, social sharing, newsletter integration
- [ ] **Advanced Features**: PWA support, offline reading, personalized recommendations

### Long-term Vision 🎯

- [ ] **AI Integration**: Content suggestions, smart search, automated tagging
- [ ] **Mobile Applications**: Native iOS and Android apps
- [ ] **Enterprise Features**: Multi-author support, analytics dashboard, API platform

See the complete [Project Plan](PLAN.md) for detailed specifications and timelines.

## License

This project is licensed under the 0BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Leptos Team** - For the excellent full-stack Rust framework
- **SurrealDB Team** - For the innovative database technology
- **Rust Community** - For the amazing ecosystem and tooling
- **DigitalOcean** - For providing hosting and deployment infrastructure

---

**Built with ❤️ using Rust, Leptos, and modern web technologies.**
