.POSIX:

## Always treat these targets as out of date
.PHONY: help rebuild build format fmt lint bloat spellcheck udeps security test install-pkgs upgrade \
	server cli build-release server-release cli-release test-report test-coverage test-coverage-html \
	test-retry test-db test-email test-migrations test-server clean-test-artifacts watch \
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
	@(printf "%s\n" \
		"--------------------------------------" \
		"|Makefile usage:| ${PROJECT}" \
		"--------------------------------------"; \
		printf "\t%s: %s\n" \
		"install-pkgs" "installs ${RUST_PKGS}" \
		"rebuild     " "rebuilds cargo for ${PROJECT}" \
		"build       " "builds cargo for ${PROJECT}" \
		"bloat       " "checks size of ${PROJECT} components" \
		"install-pkgs" "install cargo pkgs for ${PROJECT}" \
		"  *NOTE:*   " "You may optionally pass args to bloat" \
		"            " "using BLOATARGS='arg1 arg2 .. argn'" \
		"leptos      " "builds leptos components for ${PROJECT}" \
		"lint        " "lints ${PROJECT}" \
		"machete     " "check for unused deps in ${PROJECT}" \
		"outdated    " "check for outdated crates in ${PROJECT}" \
		"security    " "checks security of ${PROJECT}" \
		"sort        " "sort ${PROJECT}'s Cargo.toml" \
		"spellcheck  " "checks documentation spellcheck for ${PROJECT}" \
		"test        " "runs tests on ${PROJECT}" \
		"udeps       " "checks unused dependencies for ${PROJECT}" \
		"upgrade     " "upgrades all dependencies for ${PROJECT}" \
		"valgrind    " "run ${PROJECT} binary through valgrind")

## Define a macro for creating cargo tasks  
define DEFINE_CARGO_TASK
$(1):
	$(ECHO_PREFIX) $(2) ${PROJECT}
	@$(CARGO_MAKE_CMD) $(1)
endef

## Generic pattern rule for simple cargo-make tasks
$(eval $(call DEFINE_CARGO_TASK,rebuild,Rebuilding))
$(eval $(call DEFINE_CARGO_TASK,build,Building))
$(eval $(call DEFINE_CARGO_TASK,build-release,Building ${PROJECT} in release mode))
$(eval $(call DEFINE_CARGO_TASK,leptos,Building leptos components for))
$(eval $(call DEFINE_CARGO_TASK,fmt,Formatting))
$(eval $(call DEFINE_CARGO_TASK,lint,Linting))
$(eval $(call DEFINE_CARGO_TASK,machete,Slicing up ${PROJECT} for unused crates))
$(eval $(call DEFINE_CARGO_TASK,outdated,Checking for out-of-date deps in ${PROJECT}'s Cargo.toml))
$(eval $(call DEFINE_CARGO_TASK,security,Checking security of))
$(eval $(call DEFINE_CARGO_TASK,sort,Sorting ${PROJECT}'s Cargo.toml))
$(eval $(call DEFINE_CARGO_TASK,spellcheck,Checking spelling in documentation for))
$(eval $(call DEFINE_CARGO_TASK,udeps,Checking unused dependencies for))
$(eval $(call DEFINE_CARGO_TASK,test,Testing))
$(eval $(call DEFINE_CARGO_TASK,test-report,Running tests with detailed reporting for))
$(eval $(call DEFINE_CARGO_TASK,test-coverage,Running test coverage analysis for))
$(eval $(call DEFINE_CARGO_TASK,test-retry,Running retry mechanism tests for))
$(eval $(call DEFINE_CARGO_TASK,test-db,Running database tests for))
$(eval $(call DEFINE_CARGO_TASK,test-email,Running email tests for))
$(eval $(call DEFINE_CARGO_TASK,test-migrations,Running migration integration tests for))
$(eval $(call DEFINE_CARGO_TASK,test-server,Running server integration tests for))
$(eval $(call DEFINE_CARGO_TASK,clean-test-artifacts,Cleaning test artifacts for))
$(eval $(call DEFINE_CARGO_TASK,valgrind,Checking memory leaks for))
$(eval $(call DEFINE_CARGO_TASK,bloat,Evaluating resource allocation for))

test-coverage-html:
	$(ECHO_PREFIX) Generating HTML coverage report for ${PROJECT}
	@cargo make --makefile Makefile.toml test-coverage-html
	@echo "Coverage report available at: test-results/coverage/html/index.html"

watch:
	$(ECHO_PREFIX) Watching ${PROJECT}
	@sh db.sh&
	@cargo leptos watch

install-pkgs:
	$(ECHO_PREFIX) Installing ${RUST_PKGS}
	@rustup component add clippy rustfmt
	@cargo install ${RUST_PKGS}

upgrade:
	$(ECHO_PREFIX) Upgrading all dependencies for ${PROJECT}
	@cargo install cargo-edit
	@cargo upgrade

