.PHONY: all live review generate fmt check test validate build install install-force

all: live

live review:
	cargo run --quiet -- preview

generate:
	cargo run --quiet -- generate

fmt:
	cargo fmt --check

check:
	cargo check --all-targets

test:
	cargo test

validate: generate
	cargo run --quiet -- validate

build: fmt check test validate
	cargo build --release

install: build
	cargo run --quiet --release -- install

install-force: build
	cargo run --quiet --release -- install --force
