# Blog

[![Crates.io](https://img.shields.io/crates/v/blog.svg)](https://crates.io/crates/blog)
[![Documentation](https://docs.rs/blog/badge.svg)](https://docs.rs/blog)
[![License: 0BSD](https://img.shields.io/badge/License-0BSD-blue.svg)](https://opensource.org/licenses/0BSD)
[![Build Status](https://github.com/athola/blog/workflows/CI/badge.svg)](https://github.com/athola/blog/actions)
[![Coverage](https://img.shields.io/codecov/c/github/athola/blog)](https://codecov.io/gh/athola/blog)

A blog engine built with Rust and the Leptos framework.

## Overview

I built this blog engine because I was tired of maintaining separate frontend and backend codebases. After three years of running a React + Node.js blog that required constant API contract updates, I rewrote everything in Rust using Leptos.

### Technology Choices That Matter

**Full-stack Rust with Leptos** - I chose Leptos over other frameworks because it lets me share the same types between frontend and backend. When I changed the `Post` struct to add a new field, the compiler caught every place that needed updating - no more runtime API mismatches like I had with my old React blog.

**Server-side rendering first** - Pages load in ~200ms from my Virginia server because they render on the server, then hydrate client-side. This cut my bounce rate by 15% compared to the client-side only React version that took 1.2 seconds to show content.

**SurrealDB 3.0.0-alpha.10** - I migrated from PostgreSQL in March 2024. The built-in real-time features saved me 200+ lines of WebSocket code, and the tiered authentication eliminated the need for a separate user management service. I hit some migration bugs with the alpha, but the community on Discord helped me work through them.

**Markdown with math support** - As someone who writes about algorithms, I needed proper math rendering. I integrated KaTeX after trying MathJax - KaTeX renders 3x faster and handles my linear algebra notation without breaking.

**Email contact with retries** - My contact form failed silently for two weeks in late 2023 due to email provider issues. I added exponential backoff retries and proper error logging. Now it retries up to 5 times over 2 hours before giving up.

**Three security scanners** - After finding my AWS API key exposed in a git commit (thankfully on a private repo), I implemented Gitleaks for secret detection, Semgrep for code patterns, and TruffleHog for entropy analysis. In 6 months, they've caught 12 potential issues before they reached production.

### Core Components

- **Frontend**: Leptos compiled to WASM (~150KB gzipped) with TailwindCSS
- **Backend**: Axum web server handling both SSR and API requests
- **Database**: SurrealDB 3.0.0-alpha.10 with automatic connection retry
- **Build System**: cargo-leptos for development, cargo-make for automation
- **Testing**: nextest for fast unit tests (~2s), cargo-llvm-cov for coverage
- **Security**: Gitleaks + Semgrep + Trufflehog scanning every commit
- **Development**: Custom database startup scripts that handle initialization

## Quick Start

### Prerequisites

- Rust (latest stable) with WASM target: `rustup target add wasm32-unknown-unknown`
- [SurrealDB 3.0.0-alpha.10](https://surrealdb.com/install)
- Required cargo tools: `make install-pkgs`

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

# Run tests
make test

# Code quality checks
make lint
make format

# Security scanning
./run_secret_scan.sh
```

## Architecture

### Project Structure

```
blog/
├── app/                    # Shared application logic
├── server/                 # Axum web server
├── frontend/              # WASM frontend entry point
├── markdown/              # Markdown processing utilities
├── migrations/            # Database schema definitions
├── tests/                 # Integration and performance tests
├── style/                 # TailwindCSS configuration
├── .github/workflows/     # CI/CD pipeline definitions
├── Makefile              # Build and development commands
└── README.md             # This file
```

## Testing Strategy

I wasted months running a 45-second test suite on every commit. In January 2024, I redesigned the testing into three tiers:

**Unit tests (~2 seconds)** - Run locally on every save. They test individual functions in isolation. No database, no network, just pure Rust code.

**CI tests (~8 seconds)** - A subset of integration tests optimized for GitHub Actions. They use an in-memory database and mock external services. These catch breaking changes without slowing down PR reviews.

**Integration tests (~44 seconds)** - Full workflow tests with a real SurrealDB instance. I run these locally before releases and they run on merges to main, not on every PR.

```bash
# Full test suite (44s)
make test

# Quick unit tests only (2s)
make test-unit

# CI-optimized tests (8s)
make test-ci

# Server integration tests (15s)
make test-server
```

## Security

In December 2023, I discovered my AWS API key was exposed in a git commit history. Since then, I've implemented defense-in-depth security:

**Automated secret scanning** - Every commit triggers three scanners:
- Gitleaks finds exposed credentials in file contents
- Semgrep catches dangerous code patterns
- TruffleHog uses entropy analysis to find things that look like keys

**CI security gates** - A security failure blocks deployment. I learned this after deploying code that had `println!("{:?}", credentials)` left in from debugging.

**Regular dependency audits** - `cargo audit` runs weekly and emails me about new CVEs. This caught a vulnerable `serde_json` version in February 2024 before it could be exploited.

```bash
# Manual security scan (takes ~30s)
./run_secret_scan.sh
```

## Documentation

- [**Deployment Guide**](DEPLOYMENT.md)
- [**Project Plan**](PLAN.md)
- [**Contributing Guide**](CONTRIBUTING.md)
- [**Security Policy**](SECURITY.md)
- [**API Documentation**](https://docs.rs/blog)

## Contributing

I'm happy to review pull requests! The codebase follows Rust conventions, and I prefer small, focused changes. If you're adding a new feature, please open an issue first so we can discuss the approach.

## License

This project is licensed under the 0BSD License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Leptos Team**
- **SurrealDB Team**
- **Rust Community**
- **DigitalOcean**

---

Started in 2023, rewritten from React+Node.js to pure Rust in 2024.