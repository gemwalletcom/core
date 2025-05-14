list:
    just --list

build:
    cargo build

build-gemstone:
    just gemstone build

build-ios:
    just gemstone build-ios

install: install-typeshare install-postgres install-diesel

install-rust:
    @echo Install Rust
    @which rustup &>/dev/null || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    @source ~/.cargo/env
    @rustc --version

install-typeshare:
    @echo Install typeshare-cli
    cargo install typeshare-cli@1.13.2

install-postgres:
	brew install libpq postgresql@15
	brew link postgresql@15
	export LDFLAGS="-L/opt/homebrew/opt/libpq/lib"
	export CPPFLAGS="-I/opt/homebrew/opt/libpq/include"

install-diesel:
    @echo Install Diesel
    cargo install diesel_cli --no-default-features --features postgres --version 2.2.4 --force

test-workspace:
    cargo test --lib --workspace --quiet

test-all:
    cargo test --lib --all

test CRATE:
    cargo test --package {{CRATE}}

format:
    cargo fmt -q --all

fix:
    @cargo clippy --fix --workspace --all-targets --allow-dirty

lint:
    @cargo clippy --version
    cargo clippy -- -D warnings

unused:
    cargo install cargo-machete
    cargo machete

bloat:
    cargo install cargo-bloat --no-default-features
    cargo bloat --release --crates

migrate:
    diesel migration run

localize:
    @sh scripts/localize.sh core crates/localizer/i18n

setup-services:
    docker-compose up -d redis postgres clickhouse meilisearch

generate-ts-primitives:
    @typeshare ./crates/primitives --lang=typescript --output-file=primitives.ts 1>/dev/null 2>&1

outdated:
    cargo upgrade -i --dry-run

mod gemstone
