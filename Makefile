CARGO ?= cargo
STATE ?= idle
CANDIDATE ?= 1
FRAME ?= 0

.PHONY: setup status preview live tui start validate generate test build install install-force clean

.DEFAULT_GOAL := live

setup:
	$(CARGO) fetch

status:
	@$(CARGO) run --quiet -- status

preview:
	@$(CARGO) run --quiet -- preview --state "$(STATE)" --frame "$(FRAME)"

live:
	@$(CARGO) run --quiet -- live --candidate "$(CANDIDATE)"

tui: live

start: live

generate:
	@$(CARGO) run --quiet -- generate

validate:
	@$(CARGO) run --quiet -- validate

test:
	$(CARGO) test --all-targets

build: generate
	@$(CARGO) run --quiet -- validate
	$(CARGO) test --all-targets

install:
	$(MAKE) build
	@$(CARGO) run --quiet -- install

install-force:
	$(MAKE) build
	@$(CARGO) run --quiet -- install --force

clean:
	$(CARGO) clean
