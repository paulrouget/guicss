SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

setup:
	rustup install nightly
	rustup component add rustfmt --toolchain nightly
	cargo install cargo-doc2readme
	cargo install cargo-cranky
	cargo install cargo-udeps --locked

all: build

run:
	cargo run --example basic

build:
	cargo build --example basic

doc:
	cargo doc --no-deps

fmt:
	cargo +nightly fmt

check-fmt:
	cargo +nightly fmt --check

readme:
	cargo doc2readme --expand-macros --out Readme.md

check-readme:
	cargo doc2readme --expand-macros --out Readme.md --check

fix: fmt readme
	cargo +nightly cranky --fix

check-udeps:
	cargo +nightly udeps

check-cranky:
	cargo +nightly cranky -- -D warnings

check: doc check-readme check-fmt check-udeps check-cranky

test:
	cargo test

clean:
	rm -rf target Cargo.lock
