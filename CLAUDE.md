# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `make watch` - Start the development server with live reload and database
- `make build` - Build the project
- `make build-release` - Build for production
- `make test` - Run all tests using nextest for improved performance
- `make test-report` - Run tests with detailed JUnit XML reporting (CI profile)
- `make test-coverage` - Run tests with coverage analysis (LCOV format)
- `make test-coverage-html` - Generate HTML coverage report
- `make test-retry` - Run only retry mechanism tests
- `make test-db` - Run only database-related tests
- `make test-email` - Run only email-related tests
- `make test-migrations` - Run only migration-related tests
- `make test-server` - Run server integration tests (development server startup, routing, assets)
- `make clean-test-artifacts` - Clean test result artifacts
- `make lint` - Run clippy linting
- `make format` - Format Rust code

### Package Management
- `make install-pkgs` - Install required Cargo tools (cargo-make, cargo-audit, cargo-bloat, cargo-leptos, cargo-nextest, cargo-llvm-cov, etc.)
- `make upgrade` - Update all dependencies

### Quality Assurance
- `make security` - Run security audit
- `make outdated` - Check for outdated dependencies
- `make udeps` - Check for unused dependencies
- `make machete` - Check for unused crates
- `make spellcheck` - Check documentation spelling

### Security Scanning
- `./run_secret_scan.sh` - Run comprehensive secret scanning (Gitleaks, Semgrep, Trufflehog)
- Results saved to `secret_scanning_results/` directory
- **Current Status**: ✅ All security scans passing - no critical secrets detected

## Architecture

This is a Rust-based blog engine with the following structure:

### Workspace Layout
- **app/**: Shared application logic and components (Leptos components, routing, API types)
- **frontend/**: WASM frontend entry point for hydration
- **server/**: Axum web server with SSR support
- **markdown/**: Markdown processing utilities
- **migrations/**: Database migration definitions

### Technology Stack
- **Leptos**: Full-stack Rust web framework for UI components and SSR
- **Axum**: Backend web server framework
- **SurrealDB**: Database layer
- **Tailwind CSS**: Styling (configured via tailwind.css in style/)
- **cargo-leptos**: Build tool for Leptos applications
- **nextest**: Fast test runner with improved output and CI integration
- **cargo-llvm-cov**: Coverage analysis tool with multiple output formats

### Key Files
- `Cargo.toml`: Workspace configuration with leptos metadata
- `Makefile`/`Makefile.toml`: Build system (use `make` commands, not `cargo make` directly)
- `leptosfmt.toml`: Leptos-specific formatting configuration
- `db.sh`: Database startup script (auto-runs with `make watch`)
- `run_secret_scan.sh`: Comprehensive security scanning script
- `.gitleaksignore`: Manages false positives in secret scanning
- `.semgrep.yml`: Custom security rules for static analysis
- `.env.example`: Template for secure environment variable setup

### Development Workflow
1. Use `make watch` to start development (starts database and leptos watch automatically)
2. The site runs on 127.0.0.1:3007 with live reload on port 3001
3. Frontend code compiles to WASM, server runs with SSR
4. CSS is processed through Tailwind and served as `/pkg/blog.css`

### Leptos Configuration
- Site root: `target/site`
- Assets: `public/` directory
- Tailwind input: `style/tailwind.css`
- WASM release profile: `wasm-release` (optimized for size)
- Development environment uses debug assertions, production uses optimized builds

## CI/CD Pipeline

The project uses a comprehensive security-first GitHub Actions pipeline with the following workflows:

### Workflow Architecture
```
🔒 secrets-scan.yml (Security Gate)
    ├── 🦀 rust.yml (Build & Test)
    ├── 🗄️ migrations.yml (Database)
    └── 🚀 deploy.yml (Production - main branch only)
```

### Workflow Files
1. **`.github/workflows/secrets-scan.yml`** 
   - **Purpose**: Multi-tool security scanning (Gitleaks, Semgrep, Trufflehog)
   - **Triggers**: Push/PR to master, weekly schedule, manual dispatch
   - **Blocking**: Critical security issues prevent pipeline progression
   - **Artifacts**: 90-day retention of security scan results

2. **`.github/workflows/rust.yml`**
   - **Purpose**: Rust compilation, testing, coverage analysis
   - **Matrix**: dev/release profile builds for comprehensive testing
   - **Features**: Parallel testing, coverage reports, performance monitoring
   - **Artifacts**: Build artifacts for deployment, coverage reports

3. **`.github/workflows/migrations.yml`**
   - **Purpose**: Database migration validation and testing
   - **Features**: Syntax validation, connectivity tests, dry-run migrations
   - **Security**: Different access levels for external vs trusted PRs

4. **`.github/workflows/deploy.yml`**
   - **Purpose**: Production deployment to DigitalOcean
   - **Triggers**: Main branch pushes only (after successful security/build/migration checks)
   - **Features**: Health checks, rollback capability, comprehensive monitoring

5. **`.github/workflows/ci-cd.yml`**
   - **Purpose**: Pipeline orchestration and status reporting
   - **Features**: Workflow dependency visualization, status summaries

### Security-First Design
- **Multi-layer scanning**: Pattern-based (Gitleaks), static analysis (Semgrep), entropy-based (Trufflehog)
- **Comprehensive coverage**: Scans all file types, not just Rust code
- **Automated blocking**: Critical findings prevent deployment
- **Audit trail**: All security results retained for 90 days
- **Regular scans**: Weekly scheduled scans for ongoing security monitoring
- ✅ **Recent Fixes**: Resolved tool installation path issues in CI workflows

### Integration with Development
- **Local scanning**: Use `./run_secret_scan.sh` before committing
- **Pre-commit validation**: Security scans run on every push/PR
- **SHA hash updates**: Before committing workflow changes, update action SHA hashes to latest versions:
  ```bash
  # Example: Update actions/cache to latest v4.2.0
  # Get latest SHA: curl -s https://api.github.com/repos/actions/cache/git/refs/tags/v4.2.0
  # Replace: uses: actions/cache@OLD_SHA # v4.1.x
  # With: uses: actions/cache@1bd1e32a3bdc45362d1e726936510720a7c30a57 # v4.2.0
  ```
- **Environment protection**: `.env.example` provides secure template
- **False positive management**: `.gitleaksignore` with fingerprint-based exclusions

## Pull Request Guidelines

### Size Limits and Best Practices

**Target Size**: Pull requests should contain **2000 total lines of code changes or less** (additions + deletions combined).

#### Why Size Matters
- **Faster reviews**: Smaller PRs are reviewed more quickly and thoroughly
- **Reduced bugs**: Easier to spot issues in focused changes
- **Better discussion**: Reviewers can provide more meaningful feedback
- **Easier rollbacks**: Smaller changes are simpler to revert if needed
- **CI/CD efficiency**: Faster builds and tests with smaller changesets

#### Size Guidelines
- **✅ Ideal (0-500 lines)**: Single feature, bug fix, or refactor
- **🟡 Good (500-1500 lines)**: Medium feature or multiple related changes
- **⚠️ Large (1500-2000 lines)**: Complex feature requiring justification
- **❌ Too Large (2000+ lines)**: Should be broken into multiple PRs

#### Automated Enforcement
The project includes GitHub Actions workflow checks that:
- **Calculate total lines changed** (additions + deletions)
- **Block PRs exceeding 2000 lines** from merging
- **Provide size feedback** in PR status checks
- **Allow emergency overrides** for critical fixes (maintainer approval required)

#### Breaking Down Large Changes

**Strategies for splitting large PRs**:

1. **Feature Stages**: Break features into logical phases
   ```
   PR 1: Database schema changes
   PR 2: Backend API implementation  
   PR 3: Frontend UI components
   PR 4: Integration and testing
   ```

2. **Component Separation**: Split by architectural layers
   ```
   PR 1: Data models and types
   PR 2: Business logic functions
   PR 3: Server endpoints
   PR 4: Client-side integration
   ```

3. **Preparatory PRs**: Infrastructure first, features second
   ```
   PR 1: Dependencies and configuration
   PR 2: Helper utilities and shared code
   PR 3: Main feature implementation
   ```

#### Exception Process
For PRs that must exceed 2000 lines:
1. **Add justification** in PR description explaining why the change cannot be split
2. **Request maintainer review** for size limit override
3. **Provide detailed testing plan** showing comprehensive validation
4. **Include migration/rollback strategy** for large changes

#### Monitoring and Metrics
- PR size is tracked in CI/CD pipeline
- Regular reports on average PR size trends
- Recognition for consistently well-sized PRs

## Test Status Summary

This project maintains excellent code quality with comprehensive test coverage. All tests must pass:
- ✅ **Unit Tests**: 12/12 passing
- ✅ **Frontend Tests**: 1/1 passing  
- ✅ **Markdown Tests**: 5/5 passing
- ✅ **Server Tests**: 13/13 passing
- ✅ **Migration Core Tests**: 8/8 passing
- ✅ **Schema Evolution Tests**: 16/16 passing
- ✅ **Server Integration Tests**: 7/7 passing (recently consolidated and optimized)

### Test Quality Standards

**All tests must pass.** Failing tests indicate bugs that must be fixed before proceeding with development. The test suite includes:

1. **Unit Tests**: Core functionality validation
2. **Integration Tests**: Full system workflow verification (recently consolidated and optimized)
3. **Database Tests**: Schema migration and data integrity validation
4. **Performance Tests**: Response time and resource usage validation with CI-aware timeouts

### Recent Test Improvements

**Server Integration Test Consolidation**: The integration tests have been significantly improved:
- ✅ **Deduplicated Code**: Reduced from ~700 to ~580 lines (17% reduction)
- ✅ **Helper Functions**: Consolidated duplicate HTTP client creation and page validation logic
- ✅ **Clear Test Goals**: Each test has explicit documentation about its purpose
- ✅ **CI-Aware Testing**: Added `cfg!(coverage)` detection for extended timeouts in coverage builds
- ✅ **Structured Organization**: Tests organized by functional areas (connectivity, content, assets, performance)
- ✅ **Configuration Constants**: Test data centralized in const arrays for easy maintenance

### Verification Commands
```bash
# Run all tests - must pass 100%
make test

# Run individual test suites for focused debugging
make test-db          # Database tests
make test-email       # Email functionality tests  
make test-retry       # Retry mechanism tests
make test-server      # Server integration tests
```

## Troubleshooting

### Common Development Issues

#### 🔧 Build and Compilation Issues

**Problem**: `error: could not compile` with workspace-related errors
- **Solution**: Run `cargo clean` then `make build`
- **Cause**: Workspace dependency conflicts or stale build artifacts

**Problem**: `cargo-leptos` not found
- **Solution**: Run `make install-pkgs` to install required tools
- **Alternative**: `cargo install cargo-leptos`

**Problem**: WASM compilation fails
- **Solution**: 
  1. Check that `wasm32-unknown-unknown` target is installed: `rustup target add wasm32-unknown-unknown`
  2. Clear WASM cache: `rm -rf target/`
  3. Rebuild: `make build`

#### 🗄️ Database Issues

**Problem**: SurrealDB connection failures
- **Solution**: 
  1. Check if database is running: `pgrep -f surreal` or `ps aux | grep surreal`
  2. Restart database: `./db.sh` or check the script for correct startup command
  3. Verify connection settings in environment variables

**Problem**: Database schema/migration issues
- **Solution**:
  1. Check `migrations/` directory for schema definitions
  2. Manually run migrations if needed
  3. For development, consider recreating the database

**Problem**: "Database operation failed after retries" errors
- **Solution**:
  1. Check network connectivity to database
  2. Verify database is not overloaded
  3. Review retry configuration in `app/src/api.rs` and `server/src/utils.rs`
  4. Check database logs for specific error details

#### 🌐 Network and Connectivity Issues

**Problem**: SMTP email sending failures
- **Solution**:
  1. Verify SMTP environment variables are set: `SMTP_HOST`, `SMTP_USER`, `SMTP_PASSWORD`
  2. Check firewall/network connectivity to SMTP server
  3. Verify SMTP credentials and server settings
  4. Review retry logic in `app/src/api.rs` contact function

**Problem**: Server won't start on port 3007
- **Solution**:
  1. Check if port is in use: `lsof -i :3007` or `netstat -tulpn | grep 3007`
  2. Kill existing process: `kill -9 <PID>`
  3. Change port in Leptos configuration if needed

#### 🧪 Testing Issues

**Problem**: Tests failing with timing issues
- **Solution**: Tests use retry mechanisms with exponential backoff - may need longer timeouts in CI
- **Check**: Review test timing assertions in `app/src/api.rs` and `server/src/utils.rs`
- **Alternative**: Use `make test-report` for CI profile with extended timeouts

**Problem**: nextest not found during testing
- **Solution**: Run `make install-pkgs` to install `cargo-nextest` and `cargo-llvm-cov`
- **Alternative**: `cargo install cargo-nextest cargo-llvm-cov --locked`

**Problem**: Coverage reporting failures
- **Solution**:
  1. Ensure `llvm-tools-preview` component is installed: `rustup component add llvm-tools-preview`
  2. Check that test artifacts directory exists: `make clean-test-artifacts` then retry
  3. Verify LCOV output path: `test-results/coverage/lcov.info`

**Problem**: "No RequestUrl provided" in component tests
- **Solution**: This is expected for Leptos component tests - tests are designed to avoid full component rendering
- **Note**: Business logic should be tested separately from components per Leptos best practices

**Problem**: SurrealDB error type mismatches in tests
- **Solution**: Use proper error construction: `surrealdb::Error::Db(surrealdb::error::Db::Thrown("message"))`

**Problem**: Test output difficult to read or parse
- **Solution**: Use nextest for improved test output:
  - `make test` - Standard nextest output with better formatting
  - `make test-report` - JUnit XML for CI integration
  - `RUST_LOG=debug make test` - Verbose logging with nextest

#### 📦 Dependency and Package Issues

**Problem**: Cargo dependency resolution errors
- **Solution**:
  1. Run `make upgrade` to update dependencies
  2. Check for version conflicts in `Cargo.toml`
  3. Clear cargo cache: `rm -rf ~/.cargo/registry`

**Problem**: Missing development dependencies
- **Solution**: Add to appropriate `[dev-dependencies]` sections and ensure workspace dependencies are properly referenced

**Problem**: Outdated or vulnerable dependencies
- **Solution**: 
  1. Run `make outdated` to check for updates
  2. Run `make security` to check for vulnerabilities
  3. Update specific crates or run `make upgrade`

#### 🎨 Frontend and Styling Issues

**Problem**: CSS/Tailwind styles not loading
- **Solution**:
  1. Check that `style/tailwind.css` exists and is being processed
  2. Verify CSS is served at `/pkg/blog.css`
  3. Clear browser cache and rebuild
  4. Check Leptos configuration for CSS asset paths

**Problem**: WASM hydration failures
- **Solution**:
  1. Check browser console for JavaScript errors
  2. Verify WASM files are being served correctly
  3. Check that server and client code match (SSR/hydration mismatch)

#### 🔒 Environment and Configuration

**Problem**: Environment variables not loading
- **Solution**:
  1. Create `.env` file in project root if needed
  2. Check `dotenvy::dotenv()` is called in main
  3. Use `std::env::var()` with proper defaults

**Problem**: Development vs production configuration issues
- **Solution**:
  1. Check `cfg!(debug_assertions)` usage for environment detection
  2. Use appropriate build profiles: `make build` vs `make build-release`
  3. Verify environment-specific settings

### Debugging Tips

1. **Enable detailed logging**: Set `RUST_LOG=debug` environment variable
2. **Check server logs**: Look for retry attempts and error details
3. **Use browser dev tools**: Check network requests and console errors for frontend issues
4. **Database debugging**: Connect directly to SurrealDB to verify data and schema
5. **Incremental testing**: Use `cargo test --workspace` to run all tests, or test specific packages

### Performance Troubleshooting

**Problem**: Slow page loads or high latency
- **Solution**:
  1. Check database query performance and indexing
  2. Review retry mechanisms - too many retries can increase latency
  3. Profile with `make bloat` to check binary sizes
  4. Consider caching for frequently accessed data

**Problem**: High memory usage
- **Solution**:
  1. Profile with tools like `valgrind` (use `make valgrind` if available)
  2. Check for memory leaks in long-running database connections
  3. Review retry logic for proper resource cleanup

#### 🔐 Security and Secret Management Issues

**Problem**: Gitleaks detecting false positive secrets
- **Solution**:
  1. Review the specific finding in `secret_scanning_results/gitleaks-report.json`
  2. If confirmed false positive, add fingerprint to `.gitleaksignore`:
     ```
     filename:rule-id:line-number
     ```
  3. Re-run `./run_secret_scan.sh` to verify fix

**Problem**: Semgrep reporting configuration errors
- **Solution**:
  1. Check `.semgrep.yml` syntax with `python3 -c "import yaml; yaml.safe_load(open('.semgrep.yml'))"`
  2. Ensure regex patterns use `pattern-regex` format correctly
  3. Test specific rules: `semgrep --config=.semgrep.yml --dry-run`

**Problem**: Real secrets accidentally committed
- **Solution** (URGENT):
  1. **Immediately rotate/revoke** the exposed credential
  2. Remove from code and replace with environment variable reference
  3. Add to `.env.example` with placeholder value
  4. Run security scan to confirm resolution
  5. Consider repository history cleanup if necessary

**Problem**: Environment variables not loading in development
- **Solution**:
  1. Copy `.env.example` to `.env`: `cp .env.example .env`
  2. Fill in actual values in `.env` (never commit this file)
  3. Verify `dotenvy::dotenv()` is called in application startup
  4. Check environment variable names match exactly (case-sensitive)

**Problem**: Security scan failing in CI/CD
- **Solution**:
  1. Run locally: `./run_secret_scan.sh` to reproduce issue
  2. Check recent commits for new secrets
  3. Review changes to `.gitleaksignore` and `.semgrep.yml` 
  4. Verify all team members follow security guidelines

**Problem**: Production deployment blocked by security gate
- **Solution**:
  1. Critical security issues prevent deployment (by design)
  2. Address all findings in `secret_scanning_results/`
  3. Update security configuration files if needed
  4. Re-run security scan locally before pushing

### Security Best Practices

#### Environment Variable Security
- **Never commit secrets**: Use `.env` (git-ignored) for local development
- **Use placeholders**: `.env.example` should contain `your_api_key_here` style placeholders
- **Rotate regularly**: Change API keys and passwords periodically
- **Principle of least privilege**: Use service accounts with minimal required permissions

#### Code Security Guidelines
- **Secret references only**: Code should contain `env::var("API_KEY")`, never actual keys
- **Sanitize examples**: Documentation examples must use placeholder values
- **Review dependencies**: Run `make security` to check for vulnerable packages
- **Test security**: Always run `./run_secret_scan.sh` before committing

#### CI/CD Security Integration
- **Automatic blocking**: Security gate prevents deployment of vulnerable code
- **Regular scanning**: Weekly automated scans catch newly discovered vulnerabilities
- **Audit retention**: Security scan results kept for 90 days for compliance
- **Multi-tool approach**: Gitleaks + Semgrep + Trufflehog provide comprehensive coverage

### Getting Help

- Check recent commits for breaking changes
- Review test failures for specific error details
- Use `make lint` to catch common code issues
- Consult Leptos and Axum documentation for framework-specific issues
- Check SurrealDB documentation for database-related problems

## Development Workflows

### Common Development Patterns

#### 🔄 Adding Retry Mechanisms for Network Operations

**Use Case**: Adding resilience to database connections, API calls, or email sending

**Workflow**:
1. **Add Dependencies**: Update workspace dependencies
   ```toml
   # In Cargo.toml [workspace.dependencies]
   tokio-retry = "0.3"
   backoff = { version = "0.4", features = ["tokio"] }
   ```

2. **Add to Package Dependencies**: Include in relevant packages
   ```toml
   # In app/Cargo.toml and server/Cargo.toml
   tokio-retry.workspace = true
   backoff.workspace = true
   ```

3. **Implement Retry Logic**: Create retry helper functions
   ```rust
   use tokio_retry::{strategy::ExponentialBackoff, Retry};
   
   async fn retry_operation<F, Fut, T>(operation: F) -> Result<T, Error>
   where
       F: Fn() -> Fut,
       Fut: std::future::Future<Output = Result<T, OriginalError>>,
   {
       let retry_strategy = ExponentialBackoff::from_millis(100)
           .max_delay(Duration::from_secs(5))
           .take(3);
           
       Retry::spawn(retry_strategy, || async {
           operation().await.map_err(|e| {
               tracing::warn!("Operation failed, retrying: {:?}", e);
               e
           })
       }).await
   }
   ```

4. **Apply to Network Operations**: Wrap existing calls
   ```rust
   // Before
   let result = db.query("SELECT * FROM table").await?;
   
   // After  
   let result = retry_operation(|| async {
       db.query("SELECT * FROM table").await
   }).await?;
   ```

5. **Test the Implementation**:
   ```bash
   make test
   make lint
   ```

#### 🧪 Test-Driven Development Workflow

**Use Case**: Adding comprehensive tests for new network functionality

**Workflow**:
1. **Add Test Dependencies**: Update dev-dependencies
   ```toml
   [dev-dependencies]
   tokio-test = "0.4.4"
   mockall = "0.13.1"
   assert_matches = "1.5.0"
   ```

2. **Write Failing Tests First**: Create test cases for expected behavior
   ```rust
   #[tokio::test]
   async fn test_retry_succeeds_after_failures() {
       let call_count = Arc::new(AtomicUsize::new(0));
       // ... test implementation
   }
   ```

3. **Implement Feature**: Add minimal code to make tests pass

4. **Run Tests Continuously**:
   ```bash
   # Run specific package tests with nextest
   cargo nextest run -p app --lib
   
   # Run all tests with improved output
   make test
   
   # Run with verbose logging
   RUST_LOG=debug make test
   
   # Generate coverage during development
   make test-coverage-html
   ```

5. **Refactor and Validate**: Clean up code while keeping tests green

#### 🛠️ Fixing Build Issues Workflow

**Use Case**: Resolving compilation errors after major changes

**Workflow**:
1. **Clean Build Artifacts**:
   ```bash
   cargo clean
   rm -rf target/
   ```

2. **Check Dependencies**:
   ```bash
   make lint
   make outdated
   ```

3. **Build Incrementally**:
   ```bash
   # Check workspace structure
   cargo build --workspace --message-format=short
   
   # Build specific package
   cargo build -p app
   ```

4. **Fix Compilation Errors**: Address issues package by package
   - Start with `markdown` (fewest dependencies)
   - Then `app` (shared logic)
   - Then `frontend` and `server`

5. **Validate Fix**:
   ```bash
   make build
   make test
   ```

#### 🔧 Adding New Server Functions Workflow

**Use Case**: Adding new API endpoints with retry mechanisms

**Workflow**:
1. **Define Data Types**: Add to `app/src/types.rs`
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct NewApiRequest {
       pub field: String,
   }
   ```

2. **Create Server Function**: Add to `app/src/api.rs`
   ```rust
   #[server(endpoint = "/new-api")]
   pub async fn new_api_function(data: NewApiRequest) -> Result<String, ServerFnError> {
       let AppState { db, .. } = expect_context::<AppState>();
       
       let result = retry_db_operation(|| async {
           db.query("SELECT * FROM table").await
       }).await?;
       
       Ok("success".to_string())
   }
   ```

3. **Add Tests**: Create comprehensive test coverage
   ```rust
   #[cfg(feature = "ssr")]
   #[tokio::test]
   async fn test_new_api_function_with_retries() {
       // Test success, failure, and retry scenarios
   }
   ```

4. **Update Frontend**: Add UI components in appropriate modules

5. **Test End-to-End**:
   ```bash
   make watch  # Start dev server
   # Test in browser
   make test   # Run automated tests
   ```

#### 📦 Dependency Update Workflow

**Use Case**: Keeping dependencies current and secure

**Workflow**:
1. **Check Current Status**:
   ```bash
   make outdated      # Check for updates
   make security      # Check for vulnerabilities
   make udeps         # Check for unused deps
   ```

2. **Update Dependencies**:
   ```bash
   make upgrade       # Update all dependencies
   ```

3. **Test for Compatibility**:
   ```bash
   make build         # Check compilation
   make test          # Run tests
   make lint          # Check for new warnings
   ```

4. **Fix Breaking Changes**: Address any API changes

5. **Validate Functionality**:
   ```bash
   make watch         # Test development server
   # Manual testing of key features
   ```

#### 🐛 Debugging Network Issues Workflow

**Use Case**: Investigating retry mechanism failures or network problems

**Workflow**:
1. **Enable Debug Logging**:
   ```bash
   RUST_LOG=debug make watch
   ```

2. **Check Retry Behavior**: Look for retry attempt logs
   ```
   [WARN] Database operation failed, retrying: Connection timeout
   [INFO] Email sent successfully after 2 retries
   ```

3. **Test Network Connectivity**:
   ```bash
   # Database
   telnet localhost 8000
   
   # SMTP
   telnet smtp.example.com 587
   ```

4. **Isolate the Problem**:
   ```bash
   # Test specific components
   cargo test test_retry_db_operation
   cargo test test_email_retry
   ```

5. **Adjust Retry Configuration**: Modify timing/attempts if needed
   ```rust
   let retry_strategy = ExponentialBackoff::from_millis(200)  // Slower start
       .max_delay(Duration::from_secs(30))                    // Longer max
       .take(5);                                              // More attempts
   ```

#### 🚀 Production Deployment Workflow

**Use Case**: Preparing and deploying release builds

**Workflow**:
1. **Run Full Quality Checks**:
   ```bash
   make format        # Format code
   make lint          # Check for issues
   make security      # Security audit
   make test          # Full test suite
   ```

2. **Build for Production**:
   ```bash
   make build-release # Optimized build
   make machete       # Check for unused crates
   ```

3. **Test Production Build**:
   ```bash
   # Set production environment variables
   export DATABASE_URL=production_url
   export SMTP_HOST=production_smtp
   
   # Test production build locally
   target/release/server
   ```

4. **Prepare Deployment**:
   - Set environment variables
   - Configure reverse proxy
   - Set up database
   - Test retry mechanisms with production services

#### 🔍 Code Review Workflow

**Use Case**: Reviewing changes before merging

**Workflow**:
1. **Automated Checks**:
   ```bash
   make lint          # Code quality
   make test          # Functionality
   make security      # Security issues
   ```

2. **Manual Review Points**:
   - **Retry Logic**: Check exponential backoff configuration
   - **Error Handling**: Verify proper error conversion
   - **Test Coverage**: Ensure retry scenarios are tested
   - **Performance**: Review retry impact on latency
   - **Configuration**: Check environment variable usage

3. **Test Scenarios**:
   - Network failures (disconnect database during operation)
   - SMTP failures (invalid credentials)
   - High load (multiple concurrent retries)

#### 📊 Performance Optimization Workflow

**Use Case**: Optimizing retry mechanisms and overall performance

**Workflow**:
1. **Profile Current Performance**:
   ```bash
   make bloat         # Binary size analysis
   RUST_LOG=info make watch  # Monitor retry frequency
   ```

2. **Identify Bottlenecks**:
   - Database query performance
   - Retry attempt frequency
   - Network timeout settings

3. **Optimize Configuration**:
   ```rust
   // Faster initial retry for common transient errors
   let retry_strategy = ExponentialBackoff::from_millis(50)
       .max_delay(Duration::from_secs(2))  // Shorter for user-facing operations
       .take(3);
   ```

4. **Measure Improvements**:
   - Response time percentiles
   - Success rates after retry implementation
   - Resource usage under load

### Quick Reference Commands

```bash
# Start development
make watch

# Full development cycle with coverage
make format && make lint && make test-coverage && make build

# CI/CD pipeline commands
make test-report              # JUnit XML for CI
make test-coverage           # LCOV coverage data
make test-coverage-html      # HTML coverage report

# Dependency maintenance  
make upgrade && make security && make outdated

# Debug build issues
cargo clean && make build

# Run specific test suites
make test-retry              # Retry mechanism tests
make test-db                 # Database operation tests
make test-email              # Email functionality tests
make test-migrations         # Migration tests
make test-server             # Server integration tests

# Test with nextest directly (alternative)
cargo nextest run --workspace
cargo nextest run --workspace -- retry
RUST_LOG=debug cargo nextest run --workspace -- db

# Coverage analysis
make test-coverage-html && open test-results/coverage/html/index.html

# Clean test artifacts
make clean-test-artifacts
```

## Test Result Reporting

This project includes comprehensive test result reporting and coverage analysis tools.

### Test Reporting Features

#### JUnit XML Reports
- Generated automatically with `make test-report`
- Output location: `test-results/junit/junit.xml`
- Compatible with CI/CD systems like Jenkins, GitLab, GitHub Actions
- Includes test timing, failure details, and test grouping

#### Coverage Analysis
- **LCOV Format**: `make test-coverage` → `test-results/coverage/lcov.info`
- **HTML Reports**: `make test-coverage-html` → `test-results/coverage/html/index.html`
- **JSON Format**: Available at `test-results/coverage/coverage.json`
- Workspace-wide coverage including all packages: `app`, `server`, `frontend`, `markdown`

#### Specialized Test Suites
- **Retry Tests**: `make test-retry` - Tests retry mechanism functionality
- **Database Tests**: `make test-db` - Tests database operations and connections
- **Email Tests**: `make test-email` - Tests email functionality and SMTP operations

#### Test Organization
Tests are organized using nextest configuration:
- **Failure Output**: Immediate display of failed test details
- **Test Grouping**: Logical grouping of related tests (retry, db, email, component)
- **Parallel Execution**: Optimized test execution with nextest runner
- **Artifact Management**: Automatic cleanup with `make clean-test-artifacts`

### CI/CD Integration

#### GitHub Actions Integration

The project uses GitHub Actions workflows that trigger on pushes and pull requests to the `master` branch:

```yaml
on:
  push:
    branches: [ "master" ]
  pull_request:
    branches: [ "master" ]

# Coverage job steps:
- name: Install test tools
  run: |
    cargo install --locked cargo-nextest cargo-llvm-cov

- name: Run test coverage
  timeout-minutes: 20
  run: make test-coverage

- name: Generate HTML coverage report
  run: make test-coverage-html

- name: Upload coverage reports
  uses: actions/upload-artifact@v4
  with:
    name: coverage-reports
    path: test-results/coverage/
    retention-days: 7

- name: Upload coverage to Codecov
  if: github.event_name == 'push' || github.event_name == 'pull_request'
  uses: codecov/codecov-action@v3
  with:
    file: test-results/coverage/lcov.info
    flags: unittests
    name: codecov-umbrella
    fail_ci_if_error: false

- name: Coverage Summary
  if: github.event_name == 'push' && github.ref == 'refs/heads/master'
  run: |
    echo "✅ Coverage uploaded to Codecov for master branch"
```

#### Test Artifacts Directory Structure
```
test-results/
├── junit/
│   └── junit.xml           # JUnit XML test results
├── coverage/
│   ├── lcov.info          # LCOV coverage data
│   ├── coverage.json      # JSON coverage data
│   └── html/              # HTML coverage reports
│       └── index.html     # Main coverage report
└── reports/               # Additional test reports
```

### Coverage Configuration

Coverage reporting is configured via `.llvm-cov.toml`:
- **Package Coverage**: All workspace packages included
- **File Exclusions**: Test files automatically excluded from coverage
- **Multiple Formats**: HTML for local viewing, LCOV for CI integration
- **Workspace Support**: Unified coverage across all packages

### Development Workflow with Test Reporting

```bash
# Development cycle with nextest and coverage
make test                          # Fast tests with nextest
make test-coverage-html            # Generate viewable coverage report
open test-results/coverage/html/index.html  # View coverage

# CI/CD pipeline (matches GitHub Actions)
make clean-test-artifacts          # Clean previous results
make test-coverage                 # Generate LCOV coverage for Codecov
make test-report                   # Generate JUnit XML for CI systems

# Debugging specific test groups with nextest
make test-retry                    # Focus on retry mechanism tests
make test-db                       # Focus on database tests
make test-email                    # Focus on email functionality tests
make test-migrations               # Focus on migration tests
make test-server                   # Focus on server integration tests

# Advanced nextest usage
cargo nextest run --workspace --profile ci              # CI profile
cargo nextest run --workspace --nocapture -- retry      # With output
RUST_LOG=debug cargo nextest run --workspace -- db      # Debug logging
```

### Performance and Quality Metrics

The test reporting system provides insights into:
- **Test Execution Time**: Identify slow tests via JUnit timing data
- **Code Coverage**: Track coverage improvements across iterations
- **Test Reliability**: Monitor retry test success rates and timing
- **Component Quality**: Package-specific coverage and test results

## Linting and Code Quality

### Handling "Dead Code" Warnings

When working with test harness files, you may encounter "dead code" warnings from clippy that are false positives. This commonly occurs when:

1. Functions are defined in a test harness module but only used in separate test files
2. Clippy runs with `--all-targets` which analyzes library and test code separately
3. Conditional compilation attributes like `#[cfg(test)]` prevent functions from being compiled in certain contexts

#### Example Issue:
```
error: methods `apply_migrations`, `create_test_posts`, and `create_custom_post` are never used
   --> tests/harness/mod.rs:100:18
```

#### Solution Workflow:

1. **Verify Actual Usage**: Confirm that the flagged functions are actually used in tests:
   ```bash
   grep -r "function_name" tests/
   ```

2. **Preserve Conditional Compilation**: Keep `#[cfg(test)]` attributes for proper compilation:
   ```rust
   #[cfg(test)]
   pub async fn create_test_posts(&self, posts: &[(&str, &str, &str, &str, &str)]) -> SurrealResult<()> {
       // implementation
   }
   ```

3. **Suppress False Positives**: Add `#[allow(dead_code)]` for functions that are correctly flagged but actually used:
   ```rust
   #[allow(dead_code)]
   #[cfg(test)]
   pub async fn create_test_posts(&self, posts: &[(&str, &str, &str, &str, &str)]) -> SurrealResult<()> {
       // implementation
   }
   ```

4. **Fix Formatting Issues**: Address any clippy warnings about formatting:
   ```bash
   # Remove empty lines after doc comments
   sed -i '/^$/d' file.rs  # Remove empty lines if needed
   ```

5. **Validate Solution**:
   ```bash
   make lint     # Should pass without errors
   make test     # All tests should still pass
   ```

#### Best Practices:

- Always verify that functions are actually used before suppressing warnings
- Preserve `#[cfg(test)]` attributes to ensure proper compilation
- Use `#[allow(dead_code)]` sparingly and only when necessary
- Run both linting and tests after making changes
- Document the reasoning for any suppressed warnings in comments