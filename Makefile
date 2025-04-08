.POSIX:
## Always treat these targets as out of date
.PHONY: help rebuild build lint bloat spellcheck udeps security test install-pkgs\
	server cli build-release server-release cli-release

include .config.mk

LEPTOSARGS := build

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
		"bloat       " "checks size of ${PROJECT} components"\
		"install-pkgs" "install cargo pkgs for ${PROJECT}"\
		"  *NOTE:*   " "You may optionally pass args to bloat"\
		"            " "using BLOATARGS='arg1 arg2 .. argn'"\
		"leptos      " "builds leptos components for ${PROJECT}"\
		"lint        " "lints ${PROJECT}"\
		"machete     " "check for unused deps in ${PROJECT}"\
		"outdated    " "check for outdated crates in ${PROJECT}"\
		"security    " "checks security of ${PROJECT}"\
		"sort        " "sort ${PROJECT}'s Cargo.toml"\
		"spellcheck  " "checks documentation spellcheck for ${PROJECT}"\
		"test        " "runs tests on ${PROJECT}")\
		"udeps       " "checks unused dependencies for ${PROJECT}"\
		"valgrind    " "run ${PROJECT} binary through valgrind"

rebuild:
	@echo "[$@]: Rebuilding ${PROJECT}"
	@cargo make --makefile Makefile.toml rebuild

build:
	@echo "[$@]: Building ${PROJECT}"
	@cargo make --makefile Makefile.toml build

build-release:
	@echo "[$@]: Building ${PROJECT} in release mode"
	@cargo make --makefile Makefile.toml build-release

bloat:
	@echo "[$@]: Evaluating resource allocation of ${PROJECT}"
	@cargo make --makefile Makefile.toml bloat ${BLOATARGS}

leptos:
	@echo "[$@]: Building leptos components for ${PROJECT}"
	@cargo make --makefile Makefile.toml leptos ${LEPTOSARGS}

lint:
	@echo "[$@]: Linting ${PROJECT}"
	@cargo make --makefile Makefile.toml lint

machete:
	@echo "[$@]: Slicing up ${PROJECT} for unused crates"
	@cargo make --makefile Makefile.toml machete

outdated:
	@echo "[$@]: Checking for out-of-date deps in ${PROJECT}'s Cargo.toml"
	@cargo make --makefile Makefile.toml outdated

security:
	@echo "[$@]: Checking security of ${PROJECT}"
	@cargo make --makefile Makefile.toml security

sort:
	@echo "[$@]: Sorting ${PROJECT}'s Cargo.toml"
	@cargo make --makefile Makefile.toml sort

spellcheck:
	@echo "[$@]: Checking spelling in documentation for ${PROJECT}"
	@cargo make --makefile Makefile.toml spellcheck

udeps:
	@echo "[$@]: Checking unused dependencies for ${PROJECT}"
	@cargo make --makefile Makefile.toml udeps

test:
	@echo "[$@]: Testing ${PROJECT}"
	@cargo make --makefile Makefile.toml test

valgrind:
	@echo "[$@]: Checking memory leaks for ${PROJECT}"
	@cargo make --makefile Makefile.toml valgrind

watch:
	@echo "[$@]: Watching ${PROJECT}"
	@sh db.sh&
	@cargo leptos watch

install-pkgs:
	@echo "[$@]: Installing ${RUST_PKGS}"
	@rustup component add clippy rustfmt
	@cargo install ${RUST_PKGS}
