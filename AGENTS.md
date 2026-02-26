# AGENTS.md

Guidance for AI assistants (Claude Code, Gemini, Codex, etc.) collaborating on this repository.

## Skills

**All skills are mandatory reading** before making changes.

- [Project Structure](skills/project-structure.md) — Repo layout, crates, and tech stack
- [Development Commands](skills/development-commands.md) — Build, test, lint, format, mobile
- [Code Style](skills/code-style.md) — Formatting, naming, imports, code organization
- [Error Handling](skills/error-handling.md) — Error types, propagation, JSON access
- [Architecture](skills/architecture.md) — Provider/mapper, repository, RPC, UniFFI patterns
- [Tests](skills/tests.md) — Test conventions, mocks, integration tests
- [Defensive Programming](skills/defensive-programming.md) — Safety rules and exhaustive patterns
- [Common Issues](skills/common-issues.md) — Known anti-patterns and their fixes

## Task Completion

Before finishing a task:
1. **Review for simplification** — reduce duplication, extract helpers, consolidate modules, remove dead code
2. **Keep changes minimal** — code must be concise and focused; reviewers cannot realistically review thousands of lines per PR, so only include what is necessary for the task
3. **Run tests**: `just test <CRATE>`
4. **Run clippy**: `cargo clippy -p <crate> -- -D warnings`
5. **Format only touched files**: `rustfmt --edition 2024 <files>`
