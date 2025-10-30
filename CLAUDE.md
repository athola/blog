# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `make watch` - Start development server with live reload and database
- `make build` / `make build-release` - Build project (dev/production)
- `make test` - Run all tests using nextest (69/69 passing ✅)
- `make test-coverage` / `make test-coverage-html` - Coverage analysis
- `make test-retry` / `make test-db` / `make test-email` / `make test-migrations` / `make test-server` - Specific test suites
- `make test-ci` - CI-optimized lightweight tests
- `make test-unit` - Unit tests only
- `make test-server-integration` - Standalone server integration tests
- `make validate` - Full validation pipeline (format + lint + test + security)

### Database Management
- `make init-db` - Initialize database with users and schema (idempotent)
- `make start-db` / `make stop-db` / `make reset-db` - Database lifecycle
- `./ensure-db-ready.sh` - Database startup and initialization

### Code Quality
- `make format` / `make lint` / `make check` / `make fix` - Code formatting and linting
- `make security` / `make outdated` / `make udeps` - Security and dependency checks
- `./run_secret_scan.sh` - Secret scanning (Gitleaks + Semgrep + Trufflehog)

### Package Management
- `make install-pkgs` - Install required Cargo tools
- `make install-test-tools` - Install cargo-nextest and cargo-llvm-cov
- `make install-surrealdb` - Download and install SurrealDB locally
- `make upgrade` - Update all dependencies

## Architecture

Rust-based blog engine with Leptos frontend and Axum backend:

### Workspace Layout
- **app/**: Shared application logic (Leptos components, routing, API types)
- **frontend/**: WASM frontend entry point
- **server/**: Axum web server with SSR
- **markdown/**: Markdown processing
- **migrations/**: Database migrations

### Technology Stack
- **Leptos**: Full-stack Rust web framework
- **Axum**: Backend web server
- **SurrealDB**: Database layer (requires v3.0.0-alpha.10)
- **Tailwind CSS**: Styling
- **nextest**: Fast test runner
- **cargo-llvm-cov**: Coverage analysis

### Development Workflow
1. `make watch` starts development (database + leptos watch)
2. Site runs on 127.0.0.1:3007 with live reload on 3001
3. Frontend compiles to WASM, server runs with SSR
4. CSS processed through Tailwind and served as `/pkg/blog.css`

## CI/CD Pipeline

Security-first GitHub Actions pipeline:

```
🔒 secrets-scan.yml (Security Gate)
    ├── 🦀 rust.yml (Build & Test)
    ├── 🗄️ migrations.yml (Database)
    └── 🚀 deploy.yml (Production)
```

### Key Workflows
1. **secrets-scan.yml**: Multi-tool security scanning (Gitleaks, Semgrep, Trufflehog)
2. **rust.yml**: Compilation, testing, coverage analysis
3. **migrations.yml**: Database migration validation
4. **deploy.yml**: Production deployment to DigitalOcean
5. **pr-size-check.yml**: Comments on large PRs (2000+ lines)

### Security Features
- Multi-layer scanning blocks critical findings
- Weekly automated scans
- 90-day audit trail retention
- SHA hash updates required for workflow changes

## Test Status

All tests must pass. Current status: ✅ 69/69 passing

### Test Organization
- Unit tests, integration tests, database tests, performance tests
- Three-tier strategy: Unit (~0s) → CI-optimized (~5s) → Full integration (~44s)
- Enhanced process coordination and timeout management
- SurrealDB 3.0.0-alpha.10 compatibility fixes
- Resource-conscious testing with 50% reduction in CI resource usage

### Verification Commands
```bash
make test                    # All tests (69/69 passing)
make test-db                 # Database tests
make test-server             # Integration tests
make test-coverage-html      # Coverage report
make test-ci                 # CI-optimized tests
make test-unit               # Unit tests only
make test-server-integration # Standalone server tests
```

### Recent Test Improvements
- **SurrealDB Upgrade**: Updated to 3.0.0-alpha.10 for compatibility
- **Database Script Fixes**: Fixed db.sh log level (`--log info` instead of `--log strace`)
- **Process Coordination**: Enhanced shared server initialization and cleanup
- **Resource Optimization**: Reduced CI resource consumption by 50%
- **Three-Tier Architecture**: Unit → CI-optimized → Full integration testing

## Troubleshooting

### Common Issues

**Build Issues**:
- `cargo clean && make build` for dependency conflicts
- `make install-pkgs` for missing tools
- `rustup target add wasm32-unknown-unknown` for WASM

**Database Issues**:
- Check SurrealDB version: `surreal version` (requires 3.0.0-alpha.10)
- `make reset-db` for complete database reset
- Verify db.sh uses `--log info` not `--log strace`
- Use `./ensure-db-ready.sh` for database startup
- Check database connectivity: `curl -s http://127.0.0.1:8000/version`

**Test Issues**:
- Kill processes: `pkill -f surreal && pkill -f server`
- Check ports: `lsof -i :3007,3001,8000`
- Use debug builds for faster startup
- Set `RUN_SERVER_INTEGRATION_TESTS=1` for full integration tests
- Use `make test-ci` for lightweight CI testing

**Security Issues**:
- Add false positives to `.gitleaksignore` with fingerprints
- Run `./run_secret_scan.sh` before committing
- Rotate exposed credentials immediately
- Multi-tool scanning: Gitleaks + Semgrep + Trufflehog
- Check `secret_scanning_results/` for scan reports

### Development Patterns

#### Quick Reference
```bash
# Start development
make watch

# Full development cycle
make format && make lint && make test-coverage && make build

# Validation before commit
make validate

# Dependency maintenance
make upgrade && make security && make outdated

# Debug integration tests
cargo test --workspace --test server_integration_tests test_name -- --nocapture

# CI-optimized testing
make test-ci
```

#### Adding Features
1. Define types in `app/src/types.rs`
2. Create server functions in `app/src/api.rs`
3. Add tests for new functionality
4. Update frontend components
5. Test end-to-end with `make watch`

#### Common Workflows
- **Retry mechanisms**: Use `tokio-retry` with exponential backoff
- **Database operations**: Wrap in retry logic for resilience
- **Test-driven development**: Write tests first, implement features
- **Integration testing**: Use shared server coordination

### Environment Setup
- Copy `.env.example` to `.env` and configure
- Ensure SurrealDB 3.0.0-alpha.10 is installed and in PATH
- Run `make install-pkgs` for development tools
- Use `make init-db` for database initialization

## Best Practices

### Code Quality
- All tests must pass before proceeding
- Use `#[allow(dead_code)]` sparingly for test harness false positives
- Follow existing code conventions and patterns
- Run security scans before committing
- Update workflow SHA hashes before committing changes

### Testing Strategy
- Unit tests for core functionality
- Integration tests for full workflows
- Database tests for schema validation
- Performance tests with CI-aware timeouts
- Use debug builds for faster development testing

### Security Guidelines
- Never commit secrets - use environment variables
- Run `./run_secret_scan.sh` before commits
- Rotate credentials if accidentally exposed
- Use `.env.example` for secure templates
- Follow multi-tool scanning approach