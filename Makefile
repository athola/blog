.POSIX:

## Always treat these targets as out of date
.PHONY: help rebuild build format fmt lint bloat spellcheck udeps security test install-pkgs upgrade \\
	server cli build-release server-release cli-release test-report test-coverage test-coverage-html \\
	test-retry test-db test-email test-migrations test-server clean-test-artifacts watch \\
	install-test-tools

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
	@./db.sh & echo $$! > /tmp/test_db_pid
	@sleep 8
	@echo "  Running server integration tests..."
	@cargo test --test server_integration_tests --no-fail-fast -- --test-threads=1 || (echo "Server integration tests failed, cleaning up..." && kill `cat /tmp/test_db_pid` 2>/dev/null || true && rm -f /tmp/test_db_pid && false)
	@echo "  Cleaning up database process..."
	@kill `cat /tmp/test_db_pid` 2>/dev/null || true
	@rm -f /tmp/test_db_pid
	@echo "  Server integration tests completed successfully"

## Enhanced test target with full integration
test:
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

watch:
	$(ECHO_PREFIX) Watching $${PROJECT}
	@sh db.sh&
	@cargo leptos watch

install-pkgs:
	$(ECHO_PREFIX) Installing $${RUST_PKGS}
	@rustup component add clippy rustfmt
	@cargo install $${RUST_PKGS}

upgrade:
	$(ECHO_PREFIX) Upgrading all dependencies for $${PROJECT}
	@cargo install cargo-edit
	@cargo upgrade
