CARGO ?= cargo
STATE ?= idle
FRAME ?= 0

.PHONY: setup status preview live generate validate test build install install-force clean

setup:
	$(CARGO) fetch

status:
	@$(CARGO) run --quiet -- status

preview:
	@$(CARGO) run --quiet -- preview --state "$(STATE)" --frame "$(FRAME)"

live:
	@$(CARGO) run --quiet -- live --state "$(STATE)"

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
