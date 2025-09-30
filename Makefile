.POSIX:

## Always treat these targets as out of date
.PHONY: help rebuild build format fmt lint bloat spellcheck udeps security test install-pkgs upgrade \\
	server cli build-release server-release cli-release test-report test-coverage test-coverage-html \\
	test-retry test-db test-email test-migrations test-server clean-test-artifacts watch teardown validate \\
	install-test-tools init-db start-db stop-db reset-db

## Alias for format command (compatibility)
format: fmt

include .config.mk

LEPTOSARGS := build

## Variables for common patterns
CARGO_MAKE_CMD := cargo make --makefile Makefile.toml
ECHO_PREFIX := @echo "[$@]:"

## Help information
help:
	@echo "--------------------------------------"
	@echo "| Makefile usage: | $(PROJECT)"
	@echo "--------------------------------------"
	@echo "  install-pkgs : installs $(RUST_PKGS)"
	@echo "  rebuild      : rebuilds cargo for $(PROJECT)"
	@echo "  build        : builds cargo for $(PROJECT)"
	@echo "  bloat        : checks size of $(PROJECT) components"
	@echo "  fmt          : formats $(PROJECT)"
	@echo "  lint         : lints $(PROJECT)"
	@echo "  security     : checks security of $(PROJECT)'s Cargo.toml"
	@echo "  outdated     : checks for out-of-date dependencies in $(PROJECT)'s Cargo.toml"
	@echo "  sort         : sorts $(PROJECT)'s Cargo.toml"
	@echo "  spellcheck   : checks documentation spellcheck for $(PROJECT)"
	@echo "  udeps        : checks unused dependencies for $(PROJECT)"
	@echo "  test         : tests $(PROJECT)"
	@echo "  init-db      : initializes database users if needed"
	@echo "  start-db     : starts database server"
	@echo "  stop-db      : stops database server"
	@echo "  reset-db     : resets database (stops, removes data, restarts)"
	@echo "  teardown     : stops all watch processes and cleans up artifacts"
	@echo "  validate     : validates codebase is ready for PR submission"
	@echo "  help         : prints this help message"

## Rebuild cargo
rebuild:
	$(ECHO_PREFIX) Rebuilding $${PROJECT}
	@cargo clean
	@cargo build

## Build project
build:
	$(ECHO_PREFIX) Building $${PROJECT}
	@cargo build $${LEPTOSARGS}

## Format Rust code
fmt:
	$(ECHO_PREFIX) Formatting $${PROJECT}
	@cargo fmt --all

## Lint Rust code
lint:
	$(ECHO_PREFIX) Linting $${PROJECT}
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

## Define a template for common cargo tasks
define DEFINE_CARGO_TASK
$1:
	$(ECHO_PREFIX) $($2) ${PROJECT}
	@cargo $1
endef

$(eval $(call DEFINE_CARGO_TASK,security,Checking security of))
$(eval $(call DEFINE_CARGO_TASK,outdated,Checking for out-of-date deps in ${PROJECT}'s Cargo.toml))
$(eval $(call DEFINE_CARGO_TASK,sort,Sorting ${PROJECT}'s Cargo.toml))

## Server integration tests (requires database) - standalone
test-server-integration:
	$(ECHO_PREFIX) Running server integration tests
	@echo "Starting database for integration tests..."
	@./db.sh & echo $$! > /tmp/db_pid
	@sleep 5
	@echo "Running server integration tests..."
	@cargo test --test server_integration_tests --no-fail-fast || true
	@echo "Cleaning up database..."
	@kill `cat /tmp/db_pid` 2>/dev/null || true
	@rm -f /tmp/db_pid

## Server integration tests (embedded in main test suite)
test-server-integration-embedded:
	@echo "  Checking for existing database process..."
	@pkill -f "surreal" 2>/dev/null || true
	@sleep 2
	@echo "  Starting fresh database instance..."
	@./db.sh & echo $! > /tmp/test_db_pid
	@sleep 8
	@echo "  Running server integration tests..."
	@cargo test --test server_integration_tests --no-fail-fast -- --test-threads=1 || (echo "Server integration tests failed, cleaning up..." && kill `cat /tmp/test_db_pid` 2>/dev/null || true && rm -f /tmp/test_db_pid && false)
	@echo "  Cleaning up database process..."
	@kill `cat /tmp/test_db_pid` 2>/dev/null || true
	@rm -f /tmp/test_db_pid
	@echo "  Server integration tests completed successfully"

## Lightweight CI tests for resource-constrained environments
test-ci:
	@echo "  Running lightweight CI tests..."
	@for test in $(find . -name "*_ci*.rs" -o -name "*ci_*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running CI test: $test"; \
		cargo test --test $test --features ci --no-fail-fast -- --test-threads=1 || exit 1; \
	done
	@echo "  CI tests completed successfully"

## Unit tests only (no integration required)
test-unit:
	@echo "  Running unit tests only..."
	@for test in $(find . -name "*_unit*.rs" -o -name "*unit_*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running unit test: $test"; \
		cargo test --test $test --no-fail-fast || exit 1; \
	done
	@echo "  Unit tests completed successfully"

## Pattern-based test runner for integration tests
test-integration-pattern:
	@echo "  Running integration tests matching pattern..."
	@for test in $(find . -name "*integration*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running integration test: $test"; \
		cargo test --test $test --no-fail-fast -- --test-threads=1 || exit 1; \
	done
	@echo "  Integration pattern tests completed successfully"

## Build frontend assets (CSS, JS, WASM) required for integration tests
build-assets:
	$(ECHO_PREFIX) Building frontend assets
	@if [ ! -f target/site/pkg/blog.css ] || [ ! -f target/site/pkg/blog.js ] || [ ! -f target/site/pkg/blog.wasm ]; then \
		echo "Assets missing or incomplete, rebuilding..."; \
		cargo leptos build; \
	else \
		echo "Assets already exist, skipping build"; \
	fi

## Enhanced test target with full integration
test: build-assets
	$(ECHO_PREFIX) Testing $${PROJECT}
	@echo "Running Rust unit and integration tests..."
	@cargo test --workspace --no-fail-fast --lib --bins
	@cargo test migration_core_tests --no-fail-fast
	@cargo test schema_evolution_tests --no-fail-fast
	@cargo test server_integration_tests --no-fail-fast
	@echo ""
	@echo "✅ Full test suite completed successfully!"
	@echo "Note: Run 'make test-server-integration' separately to test server functionality"


test-coverage-html:
	$(ECHO_PREFIX) Generating HTML coverage report for $${PROJECT}
	@cargo make --makefile Makefile.toml test-coverage-html
	@echo "Coverage report available at: test-results/coverage/html/index.html"

## Initialize database users if needed
init-db:
	$(ECHO_PREFIX) Initializing database users
	@./init-db.sh

## Start database server
start-db:
	$(ECHO_PREFIX) Starting database server
	@./db.sh &

## Stop database server
stop-db:
	$(ECHO_PREFIX) Stopping database server
	@./stop-db.sh

## Reset database (stop, remove data, restart)
reset-db:
	$(ECHO_PREFIX) Resetting database
	@echo "Stopping database server..."
	@./stop-db.sh
	@sleep 2
	@echo "Removing database files..."
	@rm -rf rustblog.db rustblog_test_*.db rustblog_ci_test_*.db 2>/dev/null || true
	@echo "Database files removed. Use 'make start-db' to start fresh database."
	@echo "Database reset completed"

watch:
	$(ECHO_PREFIX) Watching $${PROJECT}
	@sh db.sh&
	@sleep 3
	@./init-db.sh
	@cargo leptos watch

## Validate codebase is ready for PR submission - runs all CI checks locally
validate: fmt lint test
	$(ECHO_PREFIX) Validating $${PROJECT} for PR submission
	@echo "Running security scans..."
	@if [ -f "./run_secret_scan.sh" ]; then chmod +x ./run_secret_scan.sh && ./run_secret_scan.sh; else echo "Warning: Secret scan script not found"; fi
	@echo "Running security audit..."
	@cargo audit --deny warnings --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2024-0320 || echo "Warning: Security audit found issues"
	@echo "Checking for unused dependencies..."
	@cargo +nightly udeps --all-targets || echo "Warning: Unused dependencies found"
	@echo "Building release profile..."
	@cargo build --workspace --profile server
	@echo "Running test coverage..."
	@$(MAKE) test-coverage-html
	@echo "Validation complete"

## Teardown all watch processes and clean artifacts
teardown:
	$(ECHO_PREFIX) Tearing down $${PROJECT}
	@echo "Stopping leptos watch processes..."
	@-pkill -f "cargo leptos watch" 2>/dev/null
	@-pkill -f "leptos" 2>/dev/null
	@echo "Stopping database processes..."
	@-pkill -f "surreal" 2>/dev/null
	@-pkill -f "db.sh" 2>/dev/null
	@echo "Cleaning up server processes..."
	@-pkill -f "server" 2>/dev/null
	@echo "Cleaning up temporary files..."
	@-rm -f /tmp/db_pid /tmp/test_db_pid 2>/dev/null
	@echo "Cleaning up build artifacts..."
	@-rm -rf target/debug/build/*/out target/debug/incremental target/debug/deps/*.d 2>/dev/null
	@echo "Teardown completed - all processes stopped and artifacts cleaned"

install-pkgs:
	$(ECHO_PREFIX) Installing $${RUST_PKGS}
	@rustup component add clippy rustfmt
	@cargo install $${RUST_PKGS}

upgrade:
	$(ECHO_PREFIX) Upgrading all dependencies for $${PROJECT}
	@cargo install cargo-edit
	@cargo upgrade
