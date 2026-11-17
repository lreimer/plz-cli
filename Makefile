CARGO ?= cargo
CARGO_TARGET_DIR ?= target

PLATFORM := $(shell uname -s)
ifeq ($(PLATFORM),Linux)
BIN_DIR ?= $(HOME)/.local/bin
else
BIN_DIR ?= $(HOME)/.bin
endif

export CARGO_TARGET_DIR

.PHONY: build debug release validate clean install install-local help

build: debug

debug:
	$(CARGO) build

release:
	$(CARGO) build --release

validate:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-features --all-targets
	$(CARGO) test

clean:
	$(CARGO) clean

install:
	$(CARGO) install --path .

# Install the release artifact without rebuilding it through cargo install.
install-local: release
	mkdir -p "$(BIN_DIR)"
	install -m 755 "$(CARGO_TARGET_DIR)/release/plz" "$(BIN_DIR)/plz"

help:
	@printf '%s\n' \
		'Usage: make [target] [VARIABLE=value]' \
		'' \
		'Targets:' \
		'  build          Build the debug binary (default target)' \
		'  debug          Build the debug binary' \
		'  release        Build the optimized release binary' \
		'  validate       Check formatting, run Clippy, and run tests' \
		'  clean          Remove Cargo build artifacts' \
		'  install        Install the CLI using cargo install --path .' \
		'  install-local  Build release and copy the binary to BIN_DIR' \
		'  help           Show this usage information' \
		'' \
		'Variables:' \
		'  BIN_DIR           Local install directory (macOS: ~/.bin; Linux: ~/.local/bin)' \
		'  CARGO             Cargo executable (default: cargo)' \
		'  CARGO_TARGET_DIR  Build artifact directory (default: target)' \
		'' \
		'Example: make install-local BIN_DIR="$$HOME/.local/bin"'
