# AGENTS.md

Guidance for AI assistants (Claude Code, Gemini, Codex, etc.) collaborating on this repository.

## Skills

Read this file first, then load the relevant skills for your current task. `project-structure.md`, `development-commands.md`, `code-style.md`, `tests.md`, and `defensive-programming.md` are the default set for most Core work. Load `error-handling.md` when touching error surfaces or JSON access, `architecture.md` when changing provider/repository/UniFFI patterns, `common-issues.md` when debugging tricky failures, and `swapper-checklist.md` only for swapper integrations.

- [Project Structure](skills/project-structure.md) — Repo layout, crates, and tech stack
- [Development Commands](skills/development-commands.md) — Build, test, lint, format, mobile
- [Code Style](skills/code-style.md) — Formatting, naming, imports, code organization
- [Error Handling](skills/error-handling.md) — Error types, propagation, JSON access
- [Architecture](skills/architecture.md) — Provider/mapper, repository, RPC, UniFFI patterns
- [Tests](skills/tests.md) — Test conventions, mocks, integration tests
- [Defensive Programming](skills/defensive-programming.md) — Safety rules and exhaustive patterns
- [Common Issues](skills/common-issues.md) — Known anti-patterns and their fixes
- [Swapper Checklist](skills/swapper-checklist.md) — Integration checklist for swapper providers

## Task Completion

Before finishing a task:
1. **Review for simplification** — reduce duplication, extract helpers, consolidate modules, remove dead code
2. **Keep changes minimal** — code must be concise and focused; reviewers cannot realistically review thousands of lines per PR, so only include what is necessary for the task
3. **Run tests**: `just test` or `just test <CRATE>`
4. **Run clippy**: `cargo clippy -p <crate> -- -D warnings`
5. **Format**: `just format`

## Test Rules

- Do not write tolerance-based assertions against live network values or values recomputed from separate RPC/API calls in integration tests. These tests are flaky and low-signal.
- For integration tests, assert stable invariants only. For exact numeric behavior, cover the pure calculation in unit tests with deterministic inputs.
- Write one test function with many assertions instead of many separate single-assertion test functions. Group related cases into a single `test_<function_name>` test.

## Testkit Mocks

- Put reusable mocks in a crate `testkit` file and attach them to the type with `impl Type { pub fn mock() -> Self }`.
- Use `mock()` for the default case; use `mock_with_*` or a clearly named variant only when needed.
- Keep mocks small, valid, and fixed. If a fixture is only used once, an inline literal is fine.

Mock example:
```rust
impl Asset {
    pub fn mock() -> Self {
        Asset::from_chain(Chain::Ethereum)
    }
}
```

Examples:
- [crates/primitives/src/testkit/asset_mock.rs](crates/primitives/src/testkit/asset_mock.rs)
- [crates/storage/src/testkit/scan_address_mock.rs](crates/storage/src/testkit/scan_address_mock.rs)
