##
# Themis Tracer
#
# @file
# @version 0.1

TEST_MD_FILES := $(wildcard tests/*.md)

build:
	cargo build

promote: $(TEST_MD_FILES)

tests/%.md: target/test-artifacts/%.md.corrected
	cp -f $< $@

make test:
	cargo test
# end
