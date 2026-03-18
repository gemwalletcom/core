# Code Style

Follow the existing code style patterns unless explicitly asked to change.

## Formatting

- Line length: 180 characters maximum (configured in `rustfmt.toml`)
- Indentation: 4 spaces (Rust standard)
- Imports: Automatically reordered with rustfmt
- Format with `just format`

## Commit Messages

Write descriptive messages following conventional commit format.

## Naming

- Files/modules: `snake_case` (e.g., `asset_id.rs`, `chain_address.rs`)
- Crates: Prefixed naming (`gem_*` for blockchains, `security_*` for security)
- Functions/variables: `snake_case`
- Structs/enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Scope-appropriate names

Inside a module, use concise names that rely on scope rather than repeating the crate/module prefix.

```rust
// bad — redundant prefix inside gem_hypercore::core_signer module
fn is_hypercore_spot_swap(order: &Order) -> bool { ... }

// good — scope already provides context
fn is_spot_swap(order: &Order) -> bool { ... }
```

### Forbidden names

Don't use `util`, `utils`, `normalize`, or any similar names for modules or functions.

### No type suffixes

Avoid type suffixes like `_str`, `_int`, `_vec` in variable names; Rust's type system makes them redundant.

```rust
// bad
let address_str = "0x1234";
let balances_vec = vec![100, 200];

// good
let address = "0x1234";
let balances = vec![100, 200];
```

### No unsolicited documentation

Don't add docstrings, comments, type annotations, or inline code comments unless explicitly asked to (including in `mod.rs` files).

## Imports

Order:
1. Standard library imports
2. External crate imports
3. Local crate imports
4. Module re-exports with `pub use`

**IMPORTANT**: Always import models and types at the top of the file. Never use inline imports inside functions. Never use full paths inline — always import types first.

```rust
// bad — inline import inside function body
fn process_data() {
    use crate::models::SomeType;
    let item = SomeType::new();
}

// bad — full path inline
fn process_data() {
    let client = storage::DatabaseClient::new(url);
}

// good — all imports at file header
use crate::models::SomeType;
use storage::DatabaseClient;

fn process_data() {
    let item = SomeType::new();
    let client = DatabaseClient::new(url);
}
```

## Code Organization

- **Modular structure**: Break down long files into smaller, focused modules by logical responsibility
- **Avoid duplication**: Search for existing implementations before writing new code; reuse existing code or crates
- **Shared crates**: If functionality could be reused, create a shared crate rather than duplicating logic
- **Bird's eye view**: Step back and look at the overall structure; identify opportunities to simplify and consolidate
- **Avoid `mut`**: Prefer immutable bindings; use `mut` only when truly necessary
- **No `#[allow(dead_code)]`**: Remove dead code instead of suppressing warnings
- **Avoid `#[serde(default)]`**: Only use when the field is genuinely optional in the API response; if the field is always present, omit it
- **Use accessor methods for enum variants**: Instead of destructuring enum variants with `match`, use typed accessor methods (e.g., `metadata.get_sequence()` instead of `match &metadata { Cosmos { sequence, .. } => ... }`)
- **No `assert!` with `contains`**: Use `assert_eq!` with concrete values; `assert!(x.contains(...))` gives useless failure messages
- **No unused fields**: Remove unused fields from structs/models; don't keep fields "for future use"
- **Constants for magic numbers**: Extract magic numbers into named constants with clear meaning
- **Minimum interface**: Don't expose unnecessary functions; if client only needs one function, don't add multiple variants
