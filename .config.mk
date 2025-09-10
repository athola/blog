## Core settings
CONFIG=.config.mk
CICONF=.gitlab-ci.yml
PROJECT?=blog

## Environment variables for cargo
export OPENSSL_NO_VENDOR := 1

## Packages
RUST_PKGS=cargo-make cargo-audit cargo-bloat cargo-leptos \
	    cargo-machete cargo-outdated cargo-sort cargo-spellcheck \
	    cargo-udeps cargo-valgrind
