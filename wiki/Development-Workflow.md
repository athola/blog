# Development Guide

This guide provides instructions for setting up and working with the blog engine locally.

## Quick Start

### Prerequisites

- Rust (latest stable) with WASM target: `rustup target add wasam32-unknown-unknown`
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

## Commands

### Development
- `make watch` - Start development server with live reload and database.
- `make build` / `make build-release` - Build project (dev/production).
- `make test` - Run all tests using nextest.
- `make test-coverage` / `make test-coverage-html` - Coverage analysis.
- `make validate` - Full validation pipeline (format + lint + test + security).

### Database Management
- `make init-db` - Initialize database with users and schema.
- `make start-db` / `make stop-db` / `make reset-db` - Database lifecycle.
- `./ensure-db-ready.sh` - Database startup and initialization.

### Code Quality
- `make format` / `make lint` / `make check` / `make fix` - Code formatting and linting.
- `make security` / `make outdated` / `make udeps` - Security and dependency checks.
- `./run_secret_scan.sh` - Secret scanning (Gitleaks, Semgrep, Trufflehog).

### Package Management
- `make install-pkgs` - Install required Cargo tools.
- `make install-surrealdb` - Download and install SurrealDB locally.
- `make upgrade` - Update all dependencies.

## Testing

All tests must pass. Current status: 69/69 passing.

### Test Organization
- Unit, integration, database, and performance tests.
- Three-tier strategy: Unit (~0s) → CI-optimized (~5s) → Full integration (~44s).

### Verification Commands
```bash
make test
make test-db
make test-server
make test-coverage-html
make test-ci
make test-unit
```

## Troubleshooting

### Common Issues

*   **Build Issues**: Run `cargo clean && make build` or `make install-pkgs`.
*   **Database Issues**: Check SurrealDB version (`surreal version`), or run `make reset-db`.
*   **Test Issues**: Kill running processes (`pkill -f surreal && pkill -f server`) and check ports (`lsof -i :3007,3001,8000`).
*   **Security Issues**: Add false positives to `.gitleaksignore`. Run `./run_secret_scan.sh` before committing.

### Development Patterns

```bash
# Start development
make watch

# Full development cycle
make format && make lint && make test-coverage && make build

# Validation before commit
make validate
```
