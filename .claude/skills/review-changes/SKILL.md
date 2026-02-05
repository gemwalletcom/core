---
name: review-changes
description: Review and fix local git changes against Gem Wallet Core coding standards and patterns
argument-hint: "[--subagent]"
disable-model-invocation: true
allowed-tools: Read, Edit, Write, Glob, Grep, Bash, Task
---

# Review and Fix Local Changes

Review uncommitted changes against the coding standards and patterns defined in this repository, then fix any issues found.

## Arguments
- `--subagent`: Run in a subagent (isolates context, runs in background)

## Mode Selection

Check if `--subagent` is in the arguments:
- If `--subagent` is present: Use the Task tool with `subagent_type: "general-purpose"` to run this review in a subagent, passing all other arguments
- If `--subagent` is NOT present: Run directly in current context (default)

## Context

Current git diff to review:
!`git diff --no-color`

Changed files:
!`git diff --name-only`

## Review Checklist

Analyze the diff above and check for the following issues:

### 1. Import Patterns
- [ ] **No inline imports**: All imports must be at the top of the file, never inside functions
- [ ] **No full paths inline**: Never use `storage::DatabaseClient::new()` inline; import types first
- [ ] **Import order**: Standard library first, then external crates, then local crates, then `pub use` re-exports

### 2. Naming Conventions
- [ ] **Files/modules**: `snake_case` (e.g., `asset_id.rs`)
- [ ] **Functions/variables**: `snake_case`
- [ ] **Structs/enums**: `PascalCase`
- [ ] **Constants**: `SCREAMING_SNAKE_CASE`
- [ ] **No generic names**: Avoid `util`, `utils`, `normalize`, or similar vague names
- [ ] **Concise helper names**: Within a module, use scope-reliant names (prefer `is_spot_swap` over `is_hypercore_spot_swap`)
- [ ] **No type suffixes**: Avoid `_str`, `_int`, `_vec` suffixes; Rust's type system makes them redundant

### 3. Error Handling
- [ ] **Prefer plain `Error`**: Use plain Error types, not `thiserror` macros
- [ ] **Implement `From` traits**: For error conversion between types
- [ ] **Propagate with `?`**: Prefer `?` operator over manual `map_err` where possible
- [ ] **Consistent `Result<T, Error>`**: Use consistent return types

### 4. Code Style
- [ ] **Line length**: Maximum 180 characters
- [ ] **Avoid `matches!`**: Don't use `matches!` for pattern matching; it's easy to miss cases later
- [ ] **No over-engineering**: Only make changes directly requested or clearly necessary
- [ ] **No docstrings/comments/annotations**: Don't add docstrings, comments, or `///` docs unless explicitly asked; remove any that were added (including in mod.rs files)
- [ ] **No `#[allow(dead_code)]`**: Remove dead code instead of suppressing warnings; if code is needed, use it
- [ ] **No unused fields**: Remove unused fields from structs/models; don't keep fields "for future use"
- [ ] **Constants for magic numbers**: Extract magic numbers into named constants with clear meaning
- [ ] **Minimum interface**: Don't expose unnecessary functions; if client only needs one function, don't add multiple variants
- [ ] **Use uniffi::remote**: For UniFFI wrapper types around external models, use `#[uniffi::remote]` instead of creating duplicate structs with `From` implementations:
  ```rust
  // Record example
  use primitives::AuthNonce;
  pub type GemAuthNonce = AuthNonce;
  #[uniffi::remote(Record)]
  pub struct GemAuthNonce { pub nonce: String, pub timestamp: u32 }

  // Enum example
  use primitives::SwapperMode;
  pub type GemSwapperMode = SwapperMode;
  #[uniffi::remote(Enum)]
  pub enum GemSwapperMode { ExactIn, ExactOut }
  ```
- [ ] **Simple solutions**: Three similar lines is better than a premature abstraction
- [ ] **Avoid `mut`**: Prefer immutable bindings; use `mut` only when truly necessary
- [ ] **Prefer one-liners**: Inline single-use variables; avoid creating variables used only once

### 5. Code Organization
- [ ] **Modular structure**: Break down files into smaller, focused modules; separate models from clients/logic (e.g., `models.rs` + `client.rs`, not everything in one file)
- [ ] **Folder modules for complexity**: When a module has multiple concerns (models, client, mappers), use a folder with `mod.rs` instead of a single file
- [ ] **Avoid duplication**: Search for existing implementations before writing new code; reuse existing code or crates
- [ ] **Shared crates**: Reusable logic belongs in shared crates (e.g., `gem_solana`, `gem_evm`), not in utility binaries; move shared code to appropriate crates
- [ ] **Bird's eye view**: Step back and identify opportunities to simplify and consolidate

### 6. Async Patterns
- [ ] **Tokio runtime**: Use `tokio` for async operations
- [ ] **Shared state**: Use `Arc<tokio::sync::Mutex<T>>` for shared async state
- [ ] **Async client structs**: Should return `Result<T, Error>`

### 7. Database Patterns
- [ ] **Separate models**: Database models should be separate from domain primitives
- [ ] **Use `as_primitive()`**: For conversion from database models
- [ ] **Repository pattern**: Access via `DatabaseClient` methods

### 8. Blockchain/RPC Patterns
- [ ] **Use `gem_jsonrpc::JsonRpcClient`**: For blockchain RPC interactions
- [ ] **Use `primitives::hex`**: For hex encoding/decoding (not `alloy_primitives::hex`)
- [ ] **U256 conversions**: Use `u256_to_biguint` and `biguint_to_u256` from `gem_evm/src/u256.rs`
- [ ] **Provider pattern**: Fetch raw data via RPC, then use mapper functions for conversion
- [ ] **Mapper files**: Place mapper functions in separate `*_mapper.rs` files

### 9. Testing
- [ ] **`#[tokio::test]`**: Use for async tests
- [ ] **Test naming**: Prefix with `test_` descriptively
- [ ] **Error handling**: Use `Result<(), Box<dyn std::error::Error + Send + Sync>>`
- [ ] **Test data**: For long JSON (>20 lines), store in `testdata/` and use `include_str!()`

### 10. Security
- [ ] **No hardcoded secrets**: Check for API keys, passwords, credentials
- [ ] **Input validation**: Validate at system boundaries (user input, external APIs)
- [ ] **OWASP top 10**: Watch for command injection, XSS, SQL injection vulnerabilities

## Workflow

Iterate at least 2-3 times to ensure all issues are caught and fixed:

### Each Iteration:
1. **Analyze**: Review the diff against the checklist
2. **Read**: Read the full content of each changed file to understand context
3. **Fix**: Apply fixes directly using the Edit tool for each issue found
4. **Format**: Run `rustfmt --edition 2024 <files>` on modified files
5. **Verify**: Run `cargo clippy -p <crate> -- -D warnings` on affected crates
6. **Check**: Review the changes again - new issues may have been introduced or revealed

### Stop when:
- No more issues are found after a full review pass
- Clippy passes with no warnings
- Code is properly formatted

## Output Format

After fixing issues, provide a summary:

1. **Issues Fixed**: List each fix made with:
   - File and line reference
   - Category (from checklist above)
   - What was changed
2. **Manual Review Needed**: Issues that require human decision (if any)
3. **Verification**: Clippy/format results

Severity levels for reporting:
- **CRITICAL**: Security issues or bugs - fix immediately
- **WARNING**: Coding standard violations - fix automatically
- **SUGGESTION**: Minor improvements - fix if straightforward, otherwise note for user
