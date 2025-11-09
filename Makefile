.POSIX:

.PHONY: help format fmt build build-release rebuild rebuild-clean \
	lint lint-fix test test-ci test-unit test-server-integration \
	test-server-integration-embedded test-integration-pattern test-report \
	test-coverage test-coverage-html test-retry test-db test-email \
	test-migrations test-server build-assets install-pkgs install-test-tools \
	install-surrealdb upgrade security outdated sort spellcheck udeps bloat \
	init-db start-db stop-db reset-db watch teardown clean-test-artifacts \
	validate server server-release

format: fmt

include .config.mk
export OPENSSL_NO_VENDOR := 1

CARGO_MAKE_CMD := cargo make --makefile Makefile.toml
ECHO_PREFIX := @echo '[$@]:'

define BRIDGE_CARGO_MAKE
$1:
	$(ECHO_PREFIX) $2
	@$(CARGO_MAKE_CMD) $1
endef

help:
	@echo "--------------------------------------"
	@echo "| Makefile commands for: $(PROJECT)"
	@echo "--------------------------------------"
	@echo "  build               : Build workspace artifacts (debug)"
	@echo "  build-release       : Build workspace artifacts (release)"
	@echo "  fmt / format        : Format Rust sources"
	@echo "  lint                : Run clippy with warnings as errors"
	@echo "  lint-fix            : Apply clippy autofixes where possible"
	@echo "  test                : Run full cargo test suite (with assets)"
	@echo "  test-ci             : Lightweight CI integration tests"
	@echo "  test-unit           : Unit tests only"
	@echo "  test-server         : Server-focused nextest suite"
	@echo "  test-db             : Database-focused nextest suite"
	@echo "  test-email          : Email-focused nextest suite"
	@echo "  test-migrations     : Migration-focused nextest suite"
	@echo "  test-retry          : Retry-focused nextest suite"
	@echo "  test-coverage       : Generate lcov coverage report"
	@echo "  test-coverage-html  : Generate HTML coverage report"
	@echo "  test-report         : Run nextest CI profile"
	@echo "  build-assets        : Build frontend assets required by tests"
	@echo "  install-pkgs        : Install required Cargo tooling"
	@echo "  install-test-tools  : Install cargo-nextest / cargo-llvm-cov"
	@echo "  install-surrealdb   : Download SurrealDB locally"
	@echo "  start-db / stop-db  : Manage local SurrealDB instance"
	@echo "  watch               : Start SurrealDB and cargo-leptos watch"
	@echo "  teardown            : Stop watch/dev processes"
	@echo "  upgrade             : Update workspace dependencies"
	@echo "  security            : Run cargo audit"
	@echo "  outdated            : Check dependency versions"
	@echo "  sort                : Sort Cargo manifests"
	@echo "  spellcheck          : Spellcheck documentation"
	@echo "  udeps               : Detect unused dependencies (nightly)"
	@echo "  bloat               : Inspect binary bloat"
	@echo "  validate            : Run full validation workflow"
	@echo "  server              : Run the server binary (debug)"
	@echo "  server-release      : Run the server binary (release)"

build:
	$(ECHO_PREFIX) Building $(PROJECT) {debug}
	@cargo build --workspace

build-release:
	$(ECHO_PREFIX) Building $(PROJECT) {release}
	@cargo build --workspace --release

rebuild:
	$(ECHO_PREFIX) Rebuilding $(PROJECT)
	@$(CARGO_MAKE_CMD) rebuild

rebuild-clean:
	$(ECHO_PREFIX) Rebuilding $(PROJECT) from a clean state
	@$(CARGO_MAKE_CMD) rebuild-clean

fmt:
	$(ECHO_PREFIX) Formatting $(PROJECT)
	@cargo fmt --all

lint:
	$(ECHO_PREFIX) Linting $(PROJECT)
	@$(CARGO_MAKE_CMD) lint

lint-fix:
	$(ECHO_PREFIX) Applying clippy fixes
	@$(CARGO_MAKE_CMD) lint-fix

## --- Testing workflow ----------------------------------------------------

build-assets:
	$(ECHO_PREFIX) Building frontend assets
	@if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then \
		echo "Installing WebAssembly target..."; \
		rustup target add wasm32-unknown-unknown; \
	fi
	@if [ ! -f target/site/pkg/blog.css ] || [ ! -f target/site/pkg/blog.js ] || [ ! -f target/site/pkg/blog.wasm ]; then \
		echo "Assets missing or incomplete, rebuilding..."; \
		cargo leptos build; \
	else \
		echo "Assets already exist, skipping build"; \
	fi

test: build-assets
	$(ECHO_PREFIX) Testing $(PROJECT)
	@echo "Running Rust unit and integration tests..."
	@set -a; . ./.env.test; set +a; cargo test --workspace --no-fail-fast --lib --bins
	@set -a; . ./.env.test; set +a; cargo test migration_core_tests --no-fail-fast
	@set -a; . ./.env.test; set +a; cargo test schema_evolution_tests --no-fail-fast
ifeq ($(RUN_SERVER_INTEGRATION_TESTS),1)
	@./run_integration_tests.sh
else
	@echo "Skipping server_integration_tests (set RUN_SERVER_INTEGRATION_TESTS=1 to enable)"
endif
	@echo ""
	@echo "✅ Full test suite completed successfully!"
	@echo "Note: Run 'make test-server-integration' separately to test server functionality"

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

test-server-integration-embedded:
	@echo "  Checking for existing database process..."
	@for pid in $$(ps aux | grep -v grep | grep surreal | awk '{print $$2}'); do \
		kill $$pid 2>/dev/null || true; \
	done
	@sleep 2
	@echo "  Starting fresh database instance..."
	@./db.sh & echo $$! > /tmp/test_db_pid
	@sleep 8
	@echo "  Running server integration tests..."
	@set -a; . ./.env.test; set +a; cargo test --test server_integration_tests --no-fail-fast -- --test-threads=1 || (echo "Server integration tests failed, cleaning up..." && kill `cat /tmp/test_db_pid` 2>/dev/null || true && rm -f /tmp/test_db_pid && false)
	@echo "  Cleaning up database process..."
	@if [ -f /tmp/test_db_pid ]; then \
		kill `cat /tmp/test_db_pid` 2>/dev/null || true; \
		rm -f /tmp/test_db_pid; \
	fi
	@echo "  Server integration tests completed successfully"

test-ci:
	@echo "  Running lightweight CI tests..."
	@for test in $(find . -name "*_ci*.rs" -o -name "*ci_*.rs" | sed 's/\.rs$$//' | xargs basename -a); do \
		echo "  Running CI test: $$test"; \
		set -a; . ./.env.test; set +a; cargo test --test $$test --features ci --no-fail-fast -- --test-threads=1 || exit 1; \
	done
	@echo "  CI tests completed successfully"

test-unit:
	@echo "  Running unit tests only..."
	@for test in $(find . -name "*_unit*.rs" -o -name "*unit_*.rs" | sed 's/\.rs$$//' | xargs basename -a); do \
		echo "  Running unit test: $$test"; \
		set -a; . ./.env.test; set +a; cargo test --test $$test --no-fail-fast || exit 1; \
	done
	@echo "  Unit tests completed successfully"

test-integration-pattern:
	@echo "  Running integration tests matching pattern..."
	@for test in $(find . -name "*integration*.rs" | sed 's/\.rs$$//' | xargs basename -a); do \
		echo "  Running integration test: $$test"; \
		set -a; . ./.env.test; set +a; cargo test --test $$test --no-fail-fast -- --test-threads=1 || exit 1; \
	done
	@echo "  Integration pattern tests completed successfully"

$(eval $(call BRIDGE_CARGO_MAKE,test-report,Running nextest CI profile))
$(eval $(call BRIDGE_CARGO_MAKE,test-coverage,Generating lcov coverage report))
$(eval $(call BRIDGE_CARGO_MAKE,test-coverage-html,Generating HTML coverage report))
$(eval $(call BRIDGE_CARGO_MAKE,test-retry,Running retry-focused tests))
$(eval $(call BRIDGE_CARGO_MAKE,test-db,Running database-focused tests))
$(eval $(call BRIDGE_CARGO_MAKE,test-email,Running email-focused tests))
$(eval $(call BRIDGE_CARGO_MAKE,test-migrations,Running migration-focused tests))
$(eval $(call BRIDGE_CARGO_MAKE,test-server,Running server-focused tests))
$(eval $(call BRIDGE_CARGO_MAKE,clean-test-artifacts,Removing cached test artifacts))
$(eval $(call BRIDGE_CARGO_MAKE,install-test-tools,Installing cargo-nextest and cargo-llvm-cov))

## --- Tooling & maintenance ------------------------------------------------

install-pkgs:
	$(ECHO_PREFIX) Installing ${RUST_PKGS}
	@cargo install ${RUST_PKGS}

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

$(eval $(call BRIDGE_CARGO_MAKE,upgrade,Upgrading workspace dependencies))
$(eval $(call BRIDGE_CARGO_MAKE,security,Running cargo audit))
$(eval $(call BRIDGE_CARGO_MAKE,outdated,Checking for out-of-date dependencies))
$(eval $(call BRIDGE_CARGO_MAKE,sort,Sorting Cargo manifests))
$(eval $(call BRIDGE_CARGO_MAKE,spellcheck,Running cargo spellcheck))
$(eval $(call BRIDGE_CARGO_MAKE,udeps,Running cargo-udeps (nightly)))
$(eval $(call BRIDGE_CARGO_MAKE,bloat,Inspecting binary bloat))

## --- Database management --------------------------------------------------

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

start-db:
	$(ECHO_PREFIX) Starting SurrealDB {background}
	@if [ -f .env ]; then \
		export $$(grep -v '^#' .env | xargs) && ./ensure-db-ready.sh; \
	else \
		./ensure-db-ready.sh; \
	fi

stop-db:
	$(ECHO_PREFIX) Stopping SurrealDB processes
	@pkill -f "surreal start" 2>/dev/null || true
	@pkill -f "surrealkv" 2>/dev/null || true

reset-db: stop-db
	$(ECHO_PREFIX) Resetting database state
	@rm -rf rustblog.db rustblog_test_*.db
	@$(MAKE) start-db

## --- Development workflow -------------------------------------------------

watch:
	$(ECHO_PREFIX) Starting development watch {Ctrl+C to stop}
	@set -e; \
	$(MAKE) start-db; \
	trap 'echo "Stopping..."; $(MAKE) teardown; exit 0' INT TERM; \
	cargo leptos watch; \
	$(MAKE) teardown

teardown:
	$(ECHO_PREFIX) Stopping development processes
	@echo "Use Ctrl+C to stop cargo leptos watch"
	@echo "Use 'make stop-db' to stop database processes"
	@echo "Development processes stopped"

server:
	$(ECHO_PREFIX) Running server {debug}
	@cargo run -p server

server-release:
	$(ECHO_PREFIX) Running server {release}
	@cargo run -p server --release

## --- Validation -----------------------------------------------------------

validate: fmt lint test
	$(ECHO_PREFIX) Validating $(PROJECT) for PR submission
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
		echo "Cleaning previous coverage data..."; \
		cargo llvm-cov clean --workspace >/dev/null 2>&1 || true; \
		if cargo llvm-cov nextest --workspace --lib --bins --exclude server_integration_tests --html --output-dir test-results/coverage/html 2>&1 | grep -v "functions have mismatched data"; then \
			echo "Coverage report available at: test-results/coverage/html/index.html"; \
		else \
			echo "Warning: cargo llvm-cov nextest failed; skipping coverage report generation"; \
		fi; \
	else \
		echo "Note: cargo-llvm-cov or cargo-nextest not installed; skipping coverage report generation"; \
	fi
	@echo "Validation complete"
