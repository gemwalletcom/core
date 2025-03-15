mod client;
#[allow(unused)]
mod clmm;
mod models;
mod provider;

pub use provider::Cetus;

// https://github.com/CetusProtocol/cetus-clmm-sui-sdk/blob/main/src/config/mainnet.ts
const CETUS_GLOBAL_CONFIG_ID: &str = "0xdaa46292632c3c4d8f31f23ea0f9b36a28ff3677e9684980e4438403a67a3d8f";
const CETUS_GLOBAL_CONFIG_SHARED_VERSION: u64 = 1574190;
const CETUS_CLMM_PACKAGE_ID: &str = "0x1eabed72c53feb3805120a081dc15963c204dc8d091542592abaf7a35689b2fb";
const CETUS_ROUTER_PACKAGE_ID: &str = "0x3a5aa90ffa33d09100d7b6941ea1c0ffe6ab66e77062ddd26320c1b073aabb10";
