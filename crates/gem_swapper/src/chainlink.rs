use num_bigint::BigInt;
use num_traits::FromBytes;

use crate::SwapperError;
use gem_evm::{
    chainlink::contract::{AggregatorInterface, CHAINLINK_ETH_USD_FEED},
    multicall3::{IMulticall3, create_call3, decode_call3_return},
};

pub struct ChainlinkPriceFeed {
    pub contract: String,
}

impl ChainlinkPriceFeed {
    pub fn new_eth_usd_feed() -> ChainlinkPriceFeed {
        ChainlinkPriceFeed {
            contract: CHAINLINK_ETH_USD_FEED.into(),
        }
    }

    pub fn latest_round_call3(&self) -> IMulticall3::Call3 {
        create_call3(&self.contract, AggregatorInterface::latestRoundDataCall {})
    }

    // Price is in 8 decimals
    pub fn decoded_answer(result: &IMulticall3::Result) -> Result<BigInt, SwapperError> {
        let decoded =
            decode_call3_return::<AggregatorInterface::latestRoundDataCall>(result).map_err(|_| SwapperError::ABIError("failed to decode answer".into()))?;
        let price = BigInt::from_le_bytes(&decoded.answer.to_le_bytes::<32>());
        Ok(price)
    }
}
