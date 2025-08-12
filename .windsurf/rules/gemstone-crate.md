---
trigger: manual
---

Gemstone is a cross platform library for iOS and Android with native async networking support, It uses mozilla/uniffi-rs heavily.


## UniFFI remote enums

When working with a type alias like `pub type GemSwapProvider = primitives::SwapProvider;`, ensure that the corresponding 1:1 mapping `#[uniffi::remote(Enum)]` code block is not removed, as it is required for compilation.

```rust
#[uniffi::remote(Enum)]
pub enum SwapperSwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    PancakeswapAptosV2,
    Thorchain,
    Jupiter,
    Across,
    Oku,
    Wagmi,
    Cetus,
    StonfiV2,
    Mayan,
    Reservoir,
    Symbiosis,
}
```

## HTTP Request

- For handling HTTP requests in Rust code that is exposed via UniFFI, use `AlienTarget`, `AlienError` and `AlienProvider` from mod
`gemstone::network`. 
- Avoid using Rust-specific HTTP clients like `reqwest` or relying directly on `tokio` runtime features within the FFI boundary. 
- The `AlienProvider` pattern delegates the actual HTTP request execution to the native platform (iOS/Android), ensuring cross-platform compatibility.
- When you work with `swapper` mod, prefer use existing `SwapperError` cases, don't add new case until it's really necessary.