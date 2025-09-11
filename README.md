# Blog

A modern, fast, and secure blog engine built with Rust using the Leptos full-stack framework.

## Features

- **Full-stack Rust**: Built with Leptos for both frontend and backend
- **Server-side rendering**: Fast initial page loads with hydration
- **Modern styling**: TailwindCSS for responsive design
- **Database-backed**: SurrealDB for data persistence
- **Markdown support**: Rich content with code highlighting and math support
- **Contact form**: Email integration with retry mechanisms
- **Security-first**: Comprehensive multi-tool security scanning (Gitleaks + Semgrep + Trufflehog)
- **Test coverage**: Extensive unit and integration testing with CI-aware optimizations
- **PR Size Management**: GitHub Actions workflow that comments on PRs with 2000+ lines changed
- **Code Quality Enforcement**: Strict linting and formatting standards with automated checks

## Technology Stack

- **Frontend**: Leptos (WASM) + TailwindCSS
- **Backend**: Axum web server with Leptos SSR
- **Database**: SurrealDB
- **Build system**: cargo-leptos
- **Testing**: nextest + cargo-llvm-cov for coverage

## Quick Start

### Prerequisites

- Rust (latest stable)
- cargo-leptos: `cargo install cargo-leptos`
- cargo-nextest: `cargo install cargo-nextest`

### Development

```bash
# Install dependencies
make install-pkgs

# Start development server with live reload
make watch

# Run tests
make test

# Check code quality
make lint
```

The development server runs on http://127.0.0.1:3007

### Production

```bash
# Build for production
make build-release

# Run security scans
./run_secret_scan.sh

# Generate test coverage
make test-coverage-html
```

## Project Structure

```
├── app/           # Shared application logic (Leptos components, API)
├── frontend/      # WASM frontend entry point
├── server/        # Axum server with SSR
├── markdown/      # Markdown processing utilities
├── migrations/    # Database schema definitions
├── tests/         # Integration tests
└── style/         # TailwindCSS configuration
```

## Development Workflow

1. **Code changes**: Edit files in `app/`, `server/`, or `frontend/`
2. **Live reload**: Changes are automatically recompiled and browser refreshes
3. **Testing**: Run `make test` to ensure all tests pass
4. **Linting**: Run `make lint` to check code quality
5. **Security**: Run `./run_secret_scan.sh` before commits

## Configuration

- **Environment**: Copy `.env.example` to `.env` and fill in values
- **Database**: SurrealDB configuration in `db.sh`
- **Styles**: TailwindCSS config in `style/tailwind.css`
- **Build**: Leptos config in `Cargo.toml` workspace metadata

## Testing

All tests must pass - failing tests indicate bugs that must be fixed:

```bash
# Run all tests
make test

# Specific test suites
make test-db          # Database tests (8/8 passing)
make test-email       # Email functionality tests
make test-retry       # Retry mechanism tests  
make test-server      # Server integration tests (7/7 passing - recently optimized for single shared server)
make test-migrations  # Migration tests (14/14 passing)

# Coverage analysis
make test-coverage-html
```

**Recent Test Improvements**:
- **Single Server Instance**: All integration tests now share a single server process, eliminating resource conflicts
- **Consolidated Integration Tests**: Reduced code duplication by 17% while maintaining full coverage
- **CI-Aware Testing**: Added `cfg!(coverage)` detection for extended timeouts in CI environments
- **Helper Function Consolidation**: Unified HTTP client creation and page validation logic
- **Structured Test Organization**: Tests organized by functional areas with clear documentation
- **Database Connection Fixes**: Resolved integration test failures by:
  - Upgrading SurrealDB to version 2.3.7 (from 2.2.2)
  - Fixing db.sh script log level (`--log trace` instead of `--log strace`)
  - Improving shared server coordination and process cleanup
- **Enhanced Process Coordination**: Improved shared server initialization and cleanup logic to prevent race conditions

## Security

The project includes comprehensive multi-tool security scanning:

- **Gitleaks**: Pattern-based secret detection in code and commits
- **Semgrep**: Static analysis with custom security rules (`.semgrep.yml`)
- **TruffleHog**: Entropy-based secret detection for comprehensive coverage
- **Cargo Audit**: Dependency vulnerability scanning
- **Automated CI Gates**: Critical findings block deployment automatically
- **False Positive Management**: Fingerprint-based ignore system (`.gitleaksignore`)

```bash
# Run comprehensive security scan locally
./run_secret_scan.sh

# Results saved to secret_scanning_results/ directory
```

**Recent Security Enhancements**:
- ✅ Fixed CI/CD workflow security tool installation paths
- ✅ Comprehensive 3-tool scanning pipeline operational
- ✅ Weekly scheduled scans for ongoing monitoring
- ✅ All critical secret exposures eliminated

## Deployment

The project uses GitHub Actions for CI/CD with security-first design:

1. **Security scan**: Multi-tool secret detection blocks unsafe deployments (recently fixed)
2. **Build and test**: Comprehensive testing with 100% pass rate across multiple environments
3. **Database validation**: Migration testing and schema validation (14/14 tests passing)
4. **Production deploy**: Automated deployment to DigitalOcean (main branch only)

**CI/CD Pipeline Architecture**:
```
🔒 secrets-scan.yml (Security Gate) ✅
    ├── 🦀 rust.yml (Build & Test) ✅  
    ├── 🗄️ migrations.yml (Database) ✅
    └── 🚀 deploy.yml (Production) ✅
```

**Recent CI/CD Improvements**:
- ✅ Fixed security tool installation path issues
- ✅ Added coverage-aware test timeouts for reliable CI testing
- ✅ Optimized test execution with single shared server instance architecture
- ✅ Enhanced workflow dependency management

## Contributing

1. Ensure all tests pass: `make test`
2. Check code quality: `make lint`  
3. Run security scan: `./run_secret_scan.sh`
4. Follow conventional commit format
5. All tests must pass before merging

## Troubleshooting

If you encounter issues, try these solutions:

### Database Connection Issues
1. Ensure SurrealDB 2.3.7 is installed (not 2.2.2)
2. Check that the db.sh script uses `--log trace` instead of `--log strace`
3. Verify database is running: `pgrep -f surreal` or `ps aux | grep surreal`
4. Restart database: `./db.sh`
5. Check port availability: `lsof -i :8000`

### Integration Test Failures
1. Kill existing processes: `pkill -f surreal && pkill -f server`
2. Clean up ports: `lsof -ti:3007,3001,8000 | xargs -r kill -9`
3. Run specific test: `cargo test --workspace --test server_integration_tests test_name`
4. Check shared server coordination logic in tests/server_integration_tests.rs

### Build Issues
1. Clean build artifacts: `cargo clean`
2. Reinstall dependencies: `make install-pkgs`
3. Check WASM target: `rustup target add wasm32-unknown-unknown`

## License

0BSD - see LICENSE file for details.
