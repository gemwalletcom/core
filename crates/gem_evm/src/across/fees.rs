pub struct LpFeeCalculator {}

pub struct RelayerFee {
    pub lower_bound: u32,
    pub upper_bound: u32,
    pub cutoff: u32,
    pub decimals: u32,
}
