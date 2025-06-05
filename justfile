default: lint

clean:
	cargo clean

test:
	cargo test --all-targets --all

lint:
	cargo clippy --all-targets

audit:
	cargo deny check all

migrate:
	cargo sqlx migrate run

build:
	cargo build

build-release:
	cargo build --release

format:
	cargo fmt --all

format-check:
	cargo fmt --check --all

check-all: format-check lint audit test

## Local Variables:
## mode: makefile
## End:
