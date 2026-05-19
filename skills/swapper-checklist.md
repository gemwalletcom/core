# Swapper Provider Integration Checklist

Audit a swapper provider in `crates/swapper/src/<provider>/provider.rs` against the standard integration checklist.

## Checklist

For the given provider, verify each item by reading the provider code and related files.

### 1. get_quote Performance
- [ ] Quote returns in under 1 second in normal conditions
- [ ] Same-chain swaps prefer onchain math and node RPC over centralized quote APIs when practical
- [ ] Cross-chain swaps may use centralized/provider APIs when that is the protocol integration surface
- [ ] Avoid unnecessary chained API/RPC calls before quoting; batch or cache calls where practical
- [ ] Errors mapped correctly (min amount, unsupported asset, etc.)

### 2. get_quote_data Correctness
- [ ] Uses `quote.from_value` (not `quote.request.value`) for input amount — these differ when `use_max_amount` is true
- [ ] Builds correct transaction for each supported chain type (EVM, Bitcoin, Solana, etc.)
- [ ] ERC20 approval checked when needed

### 3. Auto Slippage
- [ ] Slippage comes from the provider API (not hardcoded)
- [ ] Or uses a sensible default with `apply_slippage_in_bp`

### 4. Referral Fee
- [ ] Fee BPS constant defined in `crates/swapper/src/fees/mod.rs`
- [ ] Passed to the provider API in quote requests
- [ ] Referral fee token side selection is explicit and tested (prefer native/wrapped native, then stablecoins, over arbitrary route tokens)

### 5. Vault Addresses & Transaction Indexing
- [ ] `get_vault_addresses()` returns all deposit addresses (user sends to vault) and send addresses (vault sends to user) used by the provider
- [ ] Addresses match what the provider actually uses in transactions
- [ ] Deposit addresses enable `is_cross_chain_swap()` detection in `cross_chain.rs`
- [ ] Send addresses enable `is_from_vault_address()` detection for incoming swap completions
- [ ] If the provider requires memo/payload validation (like Thorchain), `is_valid_swap_transaction()` handles it

### 6. Max Swap (use_max_amount)
- [ ] `get_quote()` calls `quote_value_after_reserve_by_chain(request)?` from `crate::fees`
- [ ] Adjusted value used for both the API quote request and `Quote.from_value`
- [ ] Reserved fees for supported chains exist in `RESERVED_NATIVE_FEES` (`fees/reserve.rs`)

### 7. Swap Result Tracking
- [ ] `get_swap_result()` maps provider status to `SwapResult` / `SwapStatus`
- [ ] Handles completed, pending, and failed/refunded states

### 8. Min Amount Error Handling
- [ ] Provider-specific errors parsed into `SwapperError::InputAmountError { min_amount }`
- [ ] Min amount correctly converted to base units

### 9. Supported Assets
- [ ] `supported_assets()` returns correct `SwapperChainAsset` list
- [ ] Uses asset constants from `primitives::asset_constants` (not inline `AssetId::from_token`)

### 10. Tests
- [ ] Unit tests cover quote parsing, error mapping, and asset mapping
- [ ] Live integration tests cover provider quotes/results and are gated behind `#[cfg(feature = "swap_integration_tests")]`
- [ ] Avoid mock clients/tests that only assert mocked behavior and do not protect provider logic
- [ ] Test fixtures in `<provider>/test/` directory
