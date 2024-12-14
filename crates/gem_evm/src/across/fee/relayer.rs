use num_bigint::BigInt;

pub struct CapitalCostConfig {
    pub lower_bound: BigInt,
    pub upper_bound: BigInt,
    pub cutoff: BigInt,
    pub decimals: u32,
}

pub struct RelayerFeeCalculator {}
