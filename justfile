default: lint

clean:
	cargo clean

# Runs the whole workspace (examples included) against the default
# "postgres" feature.
test: test-postgres

test-postgres:
	cargo test --all-targets --all

# Scoped to -p georm: examples/postgres/* depends on georm's default
# ("postgres") feature via Cargo feature unification, so it must stay out of
# a --no-default-features build rather than be dragged into it.
test-sqlite:
	cargo test -p georm --no-default-features --features sqlite --all-targets

lint:
	cargo clippy --all-targets

lint-sqlite:
	cargo clippy -p georm -p georm-macros --no-default-features --features sqlite --all-targets

audit:
	cargo deny check all

migrate: migrate-postgres

migrate-postgres:
	cargo sqlx migrate run

migrate-sqlite:
	cargo sqlx migrate run --source migrations/sqlite

build:
	cargo build

build-release:
	cargo build --release

format:
	cargo fmt --all

format-check:
	cargo fmt --check --all

check-all: format-check lint audit test test-sqlite

## Local Variables:
## mode: makefile
## End:
