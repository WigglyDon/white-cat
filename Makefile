.PHONY: all live review generate fmt check test validate build install install-force

all: live

live review:
	cargo run --quiet --locked -- preview

generate:
	cargo run --quiet --locked -- generate

fmt:
	cargo fmt --check

check:
	cargo check --locked --all-targets

test:
	cargo test --locked --all-targets

validate: generate
	cargo run --quiet --locked -- validate

build: fmt check test validate
	cargo build --locked --release

install: build
	cargo run --quiet --locked --release -- install

install-force: build
	cargo run --quiet --locked --release -- install --force
