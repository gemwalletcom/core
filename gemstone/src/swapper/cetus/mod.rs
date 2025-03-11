mod client;
mod clmm;
mod models;
mod provider;

pub use provider::Cetus;

// https://github.com/CetusProtocol/cetus-clmm-sui-sdk/blob/main/src/config/mainnet.ts
// const CETUS_ROUTER_PACKAGE_ID: &str = "0x3a5aa90ffa33d09100d7b6941ea1c0ffe6ab66e77062ddd26320c1b073aabb10";
const CETUS_CLMM_PACKAGE_ID: &str = "0x1eabed72c53feb3805120a081dc15963c204dc8d091542592abaf7a35689b2fb";
const CETUS_CLMM_PUBLISHED_AT: &str = "0xc6faf3703b0e8ba9ed06b7851134bbbe7565eb35ff823fd78432baa4cbeaa12e";
const CETUS_GLOBAL_CONFIG_ID: &str = "0xdaa46292632c3c4d8f31f23ea0f9b36a28ff3677e9684980e4438403a67a3d8f";
const CETUS_INTEGRATE_PACKAGE_ID: &str = "0x996c4d9480708fb8b92aa7acf819fb0497b5ec8e65ba06601cae2fb6db3312c3";
const CETUS_INTEGRATE_PUBLISHED_AT: &str = "0x2d8c2e0fc6dd25b0214b3fa747e0fd27fd54608142cd2e4f64c1cd350cc4add4";
