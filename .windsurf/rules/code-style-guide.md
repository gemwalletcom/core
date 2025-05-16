---
trigger: always_on
---

## Code Style Guidelines

- Use Rust 2021 edition conventions
- Lint with Clippy (follow .clippy.toml)
- Error handling: Use Result types with the ? operator
- Naming: snake_case for functions/variables, CamelCase for types, "util" or "utils" are not allowed
- Testing: Use #[test] or #[tokio::test] for async tests
- Documentation: /// for function docs, //! for module docs
- Code Comments: Only adding absolute necessary comments to complex algorithm, no need for most self explanatory code
- - I'm looking at you Gemini 2.5
- Imports: Group by std, external crates, then internal modules
- Types: Always specify explicit types for public API functions
- Prefer async/await over raw futures where appropriate
- Don't change test data in unit tests to make it pass (or add hardcode values to code either)
- - I'm looking at you Claude Sonnet 3.7
