# Uniswap Swap Module

This module implements on-chain swap support for Uniswap V3 and V4 (and forks like PancakeSwap, Aerodrome, Oku, Wagmi) across multiple EVM chains.

## Architecture

The implementation is split across two crates:

### `gem_evm::uniswap` — Contract addresses and ABI encoding

Located in `core/crates/gem_evm/src/uniswap/`:

| File/Dir | Purpose |
|----------|---------|
| `deployment/mod.rs` | `Deployment` trait, Permit2 address map, provider-by-contract lookup |
| `deployment/v3.rs` | V3 deployment addresses per chain (QuoterV2 + UniversalRouter) |
| `deployment/v4.rs` | V4 deployment addresses per chain (V4Quoter + UniversalRouter) |
| `path.rs` | `BasePair` (native/stables/alternatives per chain), path encoding |
| `contracts/` | Solidity ABI types (generated via `alloy-sol-macro`) |
| `command.rs` | UniversalRouter command encoding |
| `mod.rs` | `FeeTier` enum |

### `swapper::uniswap` — Swap logic and quoting

Located in `core/crates/swapper/src/uniswap/`:

| File/Dir | Purpose |
|----------|---------|
| `v3/provider.rs` | `UniswapV3` — implements `Swapper` trait for V3 pools |
| `v4/provider.rs` | `UniswapV4` — implements `Swapper` trait for V4 pools |
| `universal_router/` | Per-DEX router configs (fee tiers, deployment lookup) |
| `swap_route.rs` | Route building (direct and two-hop) |
| `quote_result.rs` | Best-quote selection from batch RPC results |
| `fee_token.rs` | Fee token preference logic |
| `deadline.rs` | Signature deadline calculation |
| `default.rs` | Factory functions to create boxed swapper instances |

## How a Swap Works

1. **Quote request** arrives with `from_asset`, `to_asset`, `value`, and `slippage`
2. **Deployment lookup** — find QuoterV2/V4Quoter + UniversalRouter for the chain
3. **Path building** — construct direct paths (all fee tiers) and two-hop paths through intermediaries from `BasePair`
4. **Batch RPC** — call Quoter contract with all path combinations in parallel batches
5. **Best quote** — select the path with the highest output amount
6. **Swap data** — encode UniversalRouter commands with Permit2 approval if needed

### V3 vs V4 Differences

| Aspect | V3 | V4 |
|--------|----|----|
| Quoter contract | QuoterV2 | V4Quoter |
| Native token address | WETH (wrapped) | `address(0)` |
| Pool identifier | path-encoded (token+fee+token) | PoolKey struct (currency0, currency1, fee, tickSpacing, hooks) |
| Default gas limit | 500,000 | 300,000 |
| `get_base_pair` call | `weth_as_native: true` | `weth_as_native: false` |

### V3 Fork Support

V3 uses the `UniversalRouterProvider` trait to abstract over Uniswap forks:

| Provider | Chains | Fee Tiers |
|----------|--------|-----------|
| Uniswap V3 | Ethereum, Optimism, Arbitrum, Polygon, AvalancheC, Base, SmartChain, ZkSync, Celo, Blast, World, Unichain, Monad, Stable | 100, 500, 3000, 10000 |
| PancakeSwap V3 | SmartChain, OpBNB, Arbitrum, Linea, Base | 100, 500, 2500, 10000 |
| Aerodrome | Base | 100, 400, 500, 3000, 10000 |
| Oku | Sonic, Mantle, Gnosis, Plasma | 100, 500, 3000, 10000 |
| Wagmi | Sonic | 500, 1500, 3000, 10000 |

## Current V4 Chain Support

Ethereum, Optimism, Arbitrum, Polygon, AvalancheC, Base, SmartChain, Blast, World, Unichain, Celo, Monad, Ink

## How to Add a New Chain

### Prerequisites

The chain must already exist in `primitives::Chain` and `primitives::EVMChain`.

### Steps

#### 1. Add Permit2 address (`gem_evm/src/uniswap/deployment/mod.rs`)

Add the chain to `get_uniswap_permit2_by_chain`. Most chains use the standard Permit2:

```rust
Chain::Ethereum
| Chain::Optimism
| Chain::NewChain  // <-- add here
=> Some("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
```

ZkSync-family chains use a different address.

#### 2. Add deployment addresses

**For V3** (`gem_evm/src/uniswap/deployment/v3.rs`):

Add to `get_uniswap_router_deployment_by_chain`:

```rust
Chain::NewChain => Some(V3Deployment {
    quoter_v2: "0x...",        // QuoterV2 address
    permit2,
    universal_router: "0x...", // UniversalRouter address
}),
```

**For V4** (`gem_evm/src/uniswap/deployment/v4.rs`):

Add to `get_uniswap_deployment_by_chain`:

```rust
Chain::NewChain => Some(V4Deployment {
    quoter: "0x...",           // V4Quoter address
    permit2,
    universal_router: "0x...", // UniversalRouter address
}),
```

Find addresses at:
- V3: https://docs.uniswap.org/contracts/v3/reference/deployments/
- V4: https://docs.uniswap.org/contracts/v4/deployments
- All: https://github.com/Uniswap/contracts/blob/main/deployments/index.md

#### 3. Add base pairs (`gem_evm/src/uniswap/path.rs`)

In `get_base_pair`, add token addresses for the chain:

- **BTC** (WBTC equivalent) — empty string if none
- **USDC** — required for routing
- **USDT** — empty string if none

These tokens are used as intermediaries for two-hop routing (e.g., TOKEN → USDC → TOKEN).

#### 4. Verify (no code needed)

Both V3 and V4 providers auto-discover supported chains:
- V3: `get_deployment_by_chain()` returns `Some` → chain is supported
- V4: `get_uniswap_deployment_by_chain()` returns `Some` → chain is supported

The `GemSwapper` in gemstone registers both providers at startup. No additional wiring is needed.

#### 5. Test

```bash
cargo clippy -p gem_evm -p swapper -- -D warnings
cargo test -p swapper
```
