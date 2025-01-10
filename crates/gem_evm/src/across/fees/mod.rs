pub mod lp;
pub mod relayer;
pub use lp::*;
pub use relayer::*;

use alloy_primitives::U256;
use num_bigint::{BigInt, Sign};

// percent is in 18 decimals
pub fn multiply(amount: U256, percent: BigInt, decimals: u32) -> U256 {
    let amount_big = BigInt::from_bytes_le(Sign::Plus, &amount.to_le_bytes::<32>());
    // for ETH scale factor is 1
    let scale_factor = BigInt::from(10_u64.pow(18 - decimals));
    let token_decimals = BigInt::from(10_u64.pow(decimals));
    let value = amount_big * percent / scale_factor / token_decimals;
    U256::from_le_slice(&value.to_signed_bytes_le())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ether_conv::to_bn_wei;

    #[test]
    fn test_multiply() {
        let amount = U256::from(100000000_u64); // 100 USDC
        let percent = to_bn_wei("0.01", 18);
        let decimals = 6;

        let result = multiply(amount, percent, decimals);
        assert_eq!(result, U256::from(1000000));
    }
}
