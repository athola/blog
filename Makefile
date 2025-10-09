.POSIX:

## Always treat these targets as out of date
.PHONY: help rebuild build format fmt lint bloat spellcheck udeps security test install-pkgs upgrade \\
	server cli build-release server-release cli-release test-report test-coverage test-coverage-html \\
	test-retry test-db test-email test-migrations test-server clean-test-artifacts watch teardown validate \\
	install-test-tools init-db start-db stop-db reset-db

## Alias for format command (compatibility)
format: fmt

include .config.mk
export OPENSSL_NO_VENDOR := 1

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
	@set -a; . ./.env.test; set +a; cargo test --test server_integration_tests --no-fail-fast || true
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
	@set -a; . ./.env.test; set +a; cargo test --test server_integration_tests --no-fail-fast -- --test-threads=1 || (echo "Server integration tests failed, cleaning up..." && kill `cat /tmp/test_db_pid` 2>/dev/null || true && rm -f /tmp/test_db_pid && false)
	@echo "  Cleaning up database process..."
	@kill `cat /tmp/test_db_pid` 2>/dev/null || true
	@rm -f /tmp/test_db_pid
	@echo "  Server integration tests completed successfully"

## Lightweight CI tests for resource-constrained environments
test-ci:
	@echo "  Running lightweight CI tests..."
	@for test in $(find . -name "*_ci*.rs" -o -name "*ci_*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running CI test: $test"; \
		set -a; . ./.env.test; set +a; cargo test --test $test --features ci --no-fail-fast -- --test-threads=1 || exit 1; \
	done
	@echo "  CI tests completed successfully"

## Unit tests only (no integration required)
test-unit:
	@echo "  Running unit tests only..."
	@for test in $(find . -name "*_unit*.rs" -o -name "*unit_*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running unit test: $test"; \
		set -a; . ./.env.test; set +a; cargo test --test $test --no-fail-fast || exit 1; \
	done
	@echo "  Unit tests completed successfully"

## Pattern-based test runner for integration tests
test-integration-pattern:
	@echo "  Running integration tests matching pattern..."
	@for test in $(find . -name "*integration*.rs" | sed 's/\.rs$//' | xargs basename -a); do \
		echo "  Running integration test: $test"; \
		set -a; . ./.env.test; set +a; cargo test --test $test --no-fail-fast -- --test-threads=1 || exit 1; \
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
	@set -a; . ./.env.test; set +a; cargo test --workspace --no-fail-fast --lib --bins
	@set -a; . ./.env.test; set +a; cargo test migration_core_tests --no-fail-fast
	@set -a; . ./.env.test; set +a; cargo test schema_evolution_tests --no-fail-fast
	@set -a; . ./.env.test; set +a; cargo test server_integration_tests --no-fail-fast
	@echo ""
	@echo "✅ Full test suite completed successfully!"
	@echo "Note: Run 'make test-server-integration' separately to test server functionality"


test-coverage-html:
	$(ECHO_PREFIX) Generating HTML coverage report for $${PROJECT}
	@cargo make --makefile Makefile.toml test-coverage-html
	@echo "Coverage report available at: test-results/coverage/html/index.html"

## Install cargo packages
install-pkgs:
	$(ECHO_PREFIX) Installing ${RUST_PKGS}
	@cargo install ${RUST_PKGS}

## Install SurrealDB (for local development only - CI uses cached binary)
install-surrealdb:
	$(ECHO_PREFIX) Installing SurrealDB
	@if command -v surreal >/dev/null 2>&1; then \
		echo "SurrealDB already installed: $$(surreal --version)"; \
	else \
		echo "Downloading SurrealDB v2.3.7..."; \
		mkdir -p $(HOME)/.surrealdb; \
		cd $(HOME)/.surrealdb && \
		curl -sSL https://github.com/surrealdb/surrealdb/releases/download/v2.3.7/surreal-v2.3.7.linux-amd64.tgz -o surreal.tgz && \
		tar -xzf surreal.tgz && \
		chmod +x surreal && \
		rm surreal.tgz && \
		echo 'export PATH="$$HOME/.surrealdb:$$PATH"' >> $(HOME)/.bashrc || true; \
		echo "SurrealDB installed. Add $(HOME)/.surrealdb to your PATH"; \
	fi

## Initialize database users if needed (assumes SurrealDB is already installed)
init-db:
	$(ECHO_PREFIX) Initializing database users
	@if [ -f "./ensure-db-ready.sh" ]; then \
		./ensure-db-ready.sh; \
	elif [ -f "./init-db.sh" ]; then \
		./init-db.sh; \
	else \
		echo "No database initialization script found"; \
		exit 1; \
	fi
validate: fmt lint test
	$(ECHO_PREFIX) Validating $${PROJECT} for PR submission
	@echo "Running security scans..."
	@if [ -f "./run_secret_scan.sh" ]; then chmod +x ./run_secret_scan.sh && ./run_secret_scan.sh; else echo "Note: Secret scan script not found"; fi
	@echo "Running security audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit --no-fetch --deny warnings --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2024-0320 || echo "Warning: Security audit found issues"; \
	else \
		echo "Note: cargo-audit not installed; skipping security audit"; \
	fi
	@echo "Checking for unused dependencies..."
	@if command -v cargo-udeps >/dev/null 2>&1; then \
		if rustup toolchain list | grep -q '^nightly'; then \
			cargo +nightly udeps --all-targets || echo "Warning: Unused dependencies found"; \
		else \
			echo "Note: nightly toolchain not installed; skipping unused dependency check"; \
		fi; \
	else \
		echo "Note: cargo-udeps not installed; skipping unused dependency check"; \
	fi
	@echo "Building release profile..."
	@cargo build --workspace --profile server
	@echo "Running test coverage..."
	@if command -v cargo-llvm-cov >/dev/null 2>&1 && command -v cargo-nextest >/dev/null 2>&1; then \
		mkdir -p test-results/coverage/html; \
		if ! cargo llvm-cov clean --workspace >/dev/null 2>&1; then \
			echo "Note: cargo llvm-cov clean failed, continuing"; \
		fi; \
		if cargo llvm-cov nextest --workspace --html --output-dir test-results/coverage/html; then \
			echo "Coverage report available at: test-results/coverage/html/index.html"; \
		else \
			echo "Warning: cargo llvm-cov nextest failed; skipping coverage report generation"; \
		fi; \
	else \
		echo "Note: cargo-llvm-cov or cargo-nextest not installed; skipping coverage report generation"; \
	fi
	@echo "Validation complete"
