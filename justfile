mod docker

default: lint

format:
	cargo fmt --all

format-check:
	cargo fmt --check --all

build:
	cargo build

build-release:
	cargo build --release

lint:
	cargo clippy --all-targets

audit:
	cargo deny check all

test:
	cargo test --all-targets --all

coverage:
	mkdir -p coverage
	cargo tarpaulin --config .tarpaulin.local.toml

coverage-ci:
	mkdir -p coverage
	cargo tarpaulin --config .tarpaulin.ci.toml

check-all: format-check lint coverage audit

## Local Variables:
## mode: makefile
## End:
