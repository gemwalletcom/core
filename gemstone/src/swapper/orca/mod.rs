mod fee_tiers;
mod jsonrpc;
mod models;
mod provider;
mod whirlpool;
pub use provider::*;

const WHIRLPOOL_PROGRAM: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
const WHIRLPOOL_CONFIG: &str = "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ";
const FEE_TIER_DISCRIMINATOR: &str = "AR8t9QRDQXa";
