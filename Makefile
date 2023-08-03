install:
	@echo Install Rust
	@curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

install-typeshare:
	@echo Install typeshare-cli
	@cargo install typeshare-cli --version 1.6.0