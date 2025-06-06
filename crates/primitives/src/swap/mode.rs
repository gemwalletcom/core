use typeshare::typeshare;

use crate::Chain;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[typeshare]
pub enum SwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>), // supports both on-chain and cross-chain. Specify the chain for on-chain swaps
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[typeshare]
pub enum SwapMode {
    ExactIn,
    ExactOut,
}
