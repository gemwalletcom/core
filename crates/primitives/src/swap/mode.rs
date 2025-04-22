use typeshare::typeshare;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[typeshare]
pub enum SwapProviderMode {
    OnChain,
    CrossChain,
    Bridge,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[typeshare]
pub enum SwapMode {
    ExactIn,
    ExactOut,
}
