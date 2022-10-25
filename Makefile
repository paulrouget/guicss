SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

all: build

run-winit:
	cargo run --example winit

run-iced:
	cargo run --features="toolkit-iced" --example iced

build:
	cargo build --all-features --examples

doc:
	cargo doc --all-features --no-deps

fmt:
	cargo +nightly fmt

check-fmt:
	cargo +nightly fmt --check

readme:
	cargo doc2readme --expand-macros --out Readme.md

check-readme:
	cargo doc2readme --expand-macros --out Readme.md --check

fix: fmt readme
	cargo +nightly cranky --all-features --fix

check-udeps:
	cargo +nightly udeps --all-features

check-cranky:
	cargo +nightly cranky --all-features -- -D warnings

check: doc check-readme check-fmt check-udeps check-cranky

test:
	cargo test --all-features

setup:
	rustup install nightly
	rustup component add rustfmt --toolchain nightly
	cargo install cargo-doc2readme
	cargo install cargo-cranky
	cargo install cargo-udeps --locked

clean:
	rm -rf target Cargo.lock
