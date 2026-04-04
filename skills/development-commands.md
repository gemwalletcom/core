# Development Commands

All commands use the `just` task runner. Run from the workspace root unless specified.

## Build

```sh
just build                      # Build the workspace
just build-gemstone             # Build cross-platform library
just gemstone build-ios         # Build iOS Swift Package (run in gemstone/)
just gemstone build-android     # Build Android AAR (run in gemstone/)
```

## Test

```sh
just test                       # Run workspace unit tests
just test <CRATE>               # Run unit tests for a specific crate
just test-integration           # Run integration tests only
just gemstone test-ios          # Run iOS integration tests (run in gemstone/)
cargo test --test integration_test --package <CRATE> --features <FEATURE>  # Manual integration test
```

## Code Quality

```sh
just format                     # Format all code (prefer per-file below)
just lint                       # Run clippy with warnings as errors
just fix                        # Auto-fix clippy issues
just unused                     # Find unused dependencies with cargo-machete
```

**Formatting and Linting**:
```sh
just format
cargo clippy -p <crate> -- -D warnings
```

## Database

```sh
just migrate                    # Run Diesel migrations
just setup-services             # Start Docker services (PostgreSQL, Redis, Meilisearch, RabbitMQ)
```

## Mobile

```sh
just gemstone install-ios-targets       # Install iOS Rust targets (run in gemstone/)
just gemstone install-android-targets   # Install Android Rust targets and cargo-ndk (run in gemstone/)
```

Note: Mobile builds require UniFFI bindings generation and platform-specific compilation.

## Generating Bindings (After Core Code Changes)

> **IMPORTANT**: When you modify code in `gemstone/`, `swapper/`, `signer/`, or any crate that affects the mobile API, you MUST regenerate the platform bindings.

### Swift Bindings (iOS)
```sh
just gemstone bindgen-swift     # Generate Swift bindings only (run in gemstone/)
just gemstone build-ios         # Full iOS build including Swift binding generation (run in gemstone/)
```
Generated files: `gemstone/generated/swift/` → copied to `gemstone/target/spm/`

### Kotlin Bindings (Android)
```sh
just gemstone bindgen-kotlin    # Generate Kotlin bindings only (run in gemstone/)
just gemstone build-android     # Full Android build including Kotlin binding generation (run in gemstone/)
```
Generated files: `gemstone/generated/kotlin/` → copied to `gemstone/android/gemstone/src/main/java/uniffi/`

### When to Regenerate Bindings
1. After adding/modifying public functions in `gemstone/src/lib.rs`
2. After changing any UniFFI-exposed types or interfaces
3. After modifying `swapper/` or `signer/` crates that are exposed via gemstone
4. Before committing changes that affect the mobile API surface
5. When UniFFI schema or configuration changes

## Utilities

```sh
just localize                   # Update English localization files only
just localize-all               # Update all localization files
just generate-ts-primitives     # Generate TypeScript types from Rust
just outdated                   # Check for outdated dependencies
```
