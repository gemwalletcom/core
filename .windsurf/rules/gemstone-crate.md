---
trigger: manual
---

Gemstone is a cross platform library for iOS and Android with native async networking support, It uses mozilla/uniffi-rs heavily.


## UniFFI remote enums

when you work with type alias like `pub type GemSwapProvider = primitives::SwapProvider;`, don't remove the equivalent 1:1 mapping code block like, this is needed for compilation.

```rust
#[uniffi::remote(Enum)]
pub enum GemSwapProvider {
    UniswapV3,
    UniswapV4,
    PancakeswapV3,
    PancakeswapAptosV2,
    Thorchain,
    Orca,
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