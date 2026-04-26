.DEFAULT_GOAL := help

SHELL         := /bin/bash
VENV          := .venv
PYTHON_VERSION ?= python3.11

# ── Platform detection ────────────────────────────────────────────────────────

ifeq ($(OS),Windows_NT)
	VENV_BIN := $(VENV)/Scripts
else
	VENV_BIN := $(VENV)/bin
endif

# cargo produces .dylib on macOS, .so on Linux
ifeq ($(shell uname -s),Darwin)
	LIB_EXT := dylib
else
	LIB_EXT := so
endif

LIB_NAME     := libdaft_html.$(LIB_EXT)
LIB_SRC_DBG  := target/debug/$(LIB_NAME)
LIB_SRC_REL  := target/release/$(LIB_NAME)
LIB_DST      := daft_html/$(LIB_NAME)

# ── Help ──────────────────────────────────────────────────────────────────────

.PHONY: help
help:  ## Show available targets
	@grep -E '^[a-zA-Z_.-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ── Environment ───────────────────────────────────────────────────────────────

.venv:  ## Create virtual environment and install Python dependencies
	@which uv > /dev/null || (echo "Error: uv is required. Install: https://docs.astral.sh/uv/" && exit 1)
	uv venv $(VENV) -p $(PYTHON_VERSION)
	uv sync --no-install-project

# ── Toolchain ─────────────────────────────────────────────────────────────────

.PHONY: check-toolchain
check-toolchain:  ## Verify Rust toolchain matches rust-toolchain.toml
	@TOOLCHAIN="$(shell rustup show active-toolchain 2>&1)"; \
	if echo "$$TOOLCHAIN" | grep -q 'rust-toolchain.toml'; then \
		echo "Toolchain is correct, continuing with build"; \
	else \
		echo "Failed to build: Rust using incorrect toolchain: $$TOOLCHAIN"; \
		exit 1; \
	fi

# ── Build ─────────────────────────────────────────────────────────────────────

.PHONY: build
build: check-toolchain .venv  ## Compile daft-html (debug) and install into the venv
	cargo build
	cp $(LIB_SRC_DBG) $(LIB_DST)
	uv pip install -e . --quiet

.PHONY: build-release
build-release: check-toolchain .venv  ## Compile daft-html (optimised release) and install into the venv
	cargo build --release
	cp $(LIB_SRC_REL) $(LIB_DST)
	uv pip install -e . --quiet

# ── Test ──────────────────────────────────────────────────────────────────────

.PHONY: test
test: .venv  ## Run the test suite
	$(VENV_BIN)/pytest tests/ -v

.PHONY: build-whl
build-whl: check-toolchain .venv  ## Build distributable wheel (tag + artifact copy handled by hatch_build.py)
	cargo build --release
	$(VENV_BIN)/python -m build --wheel --no-isolation

# ── Clean ─────────────────────────────────────────────────────────────────────

.PHONY: clean
clean:  ## Remove Cargo build artifacts and the copied native library
	cargo clean
	rm -f $(LIB_DST)

.PHONY: clean-all
clean-all: clean  ## Remove Cargo artifacts, virtual env, and packaging outputs
	rm -rf $(VENV) dist/ daft_html.egg-info/
