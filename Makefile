##
# Themis Tracer
#
# @file
# @version 0.1

TEST_MD_FILES := $(wildcard tests/*.md)

PHONY: test build lint ci

all: build test

build:
	cargo build

promote: $(TEST_MD_FILES)

tests/%.md: target/test-artifacts/%.md.corrected
	cp -f $< $@

ci: lint test

lint:
	cargo clippy -- -D warnings

test:
	cargo check --features "strict"
	cargo fmt --all -- --check
	cargo test
# end
