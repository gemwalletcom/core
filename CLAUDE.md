# Gem Core Codebase Guide

## Build/Test/Lint Commands
- Build: `just build` or `just gemstone build`
- Lint: `just lint` or `just format` 
- Test all: `just test-all` or `just test-workspace`
- Test specific crate: `just test CRATE` (e.g., `just test gemstone`)
- Run single test: `cargo test test_name` or `just gemstone test test_name`
- Integration tests: `just gemstone integration-test`
- Platform-specific: `just build-ios`, `just gemstone test-ios`

## Code Style Guidelines
- Use Rust 2021 edition conventions
- Max line length: 160 characters (per rustfmt.toml)
- Error handling: Use Result types with the ? operator
- Naming: snake_case for functions/variables, CamelCase for types
- Testing: Use #[test] or #[tokio::test] for async tests
- Documentation: /// for function docs, //! for module docs
- Imports: Group by std, external crates, then internal modules
- Types: Always specify explicit types for public API functions
- Prefer async/await over raw futures where appropriate