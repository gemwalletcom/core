install: install-rust install-typeshare install-diesel
	@echo Install Rust
	@curl curl https://sh.rustup.rs -sSf | sh -s -- -y

install-rust:
	@echo Install Rust
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	@cargo install cargo-machete

install-typeshare:
	@echo Install typeshare-cli
	@cargo install typeshare-cli --version 1.11.0 --force

install-diesel:
	brew install libpq postgresql@15
	brew link postgresql@15
	export LDFLAGS="-L/opt/homebrew/opt/libpq/lib"
	export CPPFLAGS="-I/opt/homebrew/opt/libpq/include"
	cargo install diesel_cli --no-default-features --features postgres --version 2.2.0 --force

test:
	cargo test --workspace --quiet

format:
	cargo fmt -q --all

lint:
	cargo clippy --version
	cargo clippy -- -D warnings
fix:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --quiet

unused:
	cargo machete

migrate:
	diesel migration run

localize:
	@sh scripts/localize.sh core crates/localizer/i18n
