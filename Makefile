install: install-rust install-typeshare install-diesel
	@echo Install Rust
	@curl curl https://sh.rustup.rs -sSf | sh -s -- -y

install-rust:
	@echo Install Rust
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

install-typeshare:
	@echo Install typeshare-cli
	@cargo install typeshare-cli --version 1.6.0

install-diesel:
	brew install libpq postgresql@15
	brew link postgresql@15
	export LDFLAGS="-L/opt/homebrew/opt/libpq/lib"
	export CPPFLAGS="-I/opt/homebrew/opt/libpq/include"
	cargo install diesel_cli --no-default-features --features postgres

test:
	cargo test --workspace --quiet

format:
	cargo fmt -q --all

fix:
	cargo clippy --fix --allow-dirty --allow-staged --workspace --quiet

unused:
	cargo machete

migrate:
	diesel migration run
