SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

all: build

run:
	cargo run --example basic

build:
	cargo build --example basic

doc:
	cargo doc --no-deps

fmt:
	rustup run nightly cargo fmt

check-fmt:
	rustup run nightly cargo fmt --check

readme:
	cargo readme > Readme.md

fix: fmt readme
	rustup run nightly cargo cranky --fix

check-udeps:
	rustup run nightly cargo udeps

check-cranky:
	rustup run nightly cargo cranky -- -D warnings

check: test doc check-fmt check-udeps check-cranky
	cargo readme > /dev/null

test:
	cargo test

clean:
	rm -rf target Cargo.lock
