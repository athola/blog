SIX:

## Always treat these targets as out of date
.PHONY: help rebuild build lint bloat spellcheck udeps security test install-pkgs\
	server cli build-release server-release cli-release

include .config.mk

## Help information
help:
	@(printf "%s\n"\
		"--------------------------------------"\
		"|Makefile usage:| ${PROJECT}"\
		"--------------------------------------";\
		printf "\t%s: %s\n"\
		"install-pkgs" "installs ${RUST_PKGS}"\
		"rebuild     " "rebuilds cargo for ${PROJECT}"\
		"build       " "builds cargo for ${PROJECT}"\
		"lint        " "lints ${PROJECT}"\
		"bloat       " "checks size of ${PROJECT} components"\
		"  *NOTE:*   " "You may optionally pass args to bloat"\
		"            " "using BLOATARGS='arg1 arg2 .. argn'"\
		"spellcheck  " "checks documentation spellcheck for ${PROJECT}"\
		"udeps       " "checks unused dependencies for ${PROJECT}"\
		"security    " "checks security of ${PROJECT}"\
		"test        " "runs tests on ${PROJECT}")

rebuild:
	@echo "[$@]: Rebuilding ${PROJECT}"
	@cargo make --makefile Makefile.toml rebuild

build:
	@echo "[$@]: Building ${PROJECT}"
	@cargo make --makefile Makefile.toml build

build-release:
	@echo "[$@]: Building ${PROJECT} in release mode"
	@cargo make --makefile Makefile.toml build-release

lint:
	@echo "[$@]: Linting ${PROJECT}"
	@cargo make --makefile Makefile.toml lint

security:
	@echo "[$@]: Checking security of ${PROJECT}"
	@cargo make --makefile Makefile.toml security

bloat:
	@echo "[$@]: Evaluating resource allocation of ${PROJECT}"
	@cargo make --makefile Makefile.toml bloat ${BLOATARGS}

spellcheck:
	@echo "[$@]: Checking spelling in documentation for ${PROJECT}"
	@cargo make --makefile Makefile.toml spellcheck

udeps:
	@echo "[$@]: Checking unused dependencies for ${PROJECT}"
	@cargo make --makefile Makefile.toml udeps

valgrind:
	@echo "[$@]: Checking memory leaks for ${PROJECT}"
	@cargo make --makefile Makefile.toml valgrind

test:
	@echo "[$@]: Testing ${PROJECT}"
	@cargo make --makefile Makefile.toml test

install-pkgs:
	@echo "[$@]: Installing ${RUST_PKGS}"
	@rustup component add clippy
	@rustup component add rustfmt
	@cargo install cargo-make
	@cargo install cargo-audit
	@cargo install cargo-bloat
	@cargo install cargo-spellcheck
	@cargo install cargo-udeps
	@cargo install cargo-valgrind
