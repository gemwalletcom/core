use std::collections::HashSet;

use super::FeeTier;
use crate::address::EthereumAddress;

use alloy_core::primitives::Bytes;
use alloy_primitives::aliases::U24;
use primitives::{AssetId, EVMChain};

pub struct TokenPair {
    pub token_in: EthereumAddress,
    pub token_out: EthereumAddress,
    pub fee_tier: FeeTier,
}

impl TokenPair {
    pub fn new_two_hop(token_in: &EthereumAddress, intermediary: &EthereumAddress, token_out: &EthereumAddress, fee_tier: &FeeTier) -> Vec<TokenPair> {
        vec![
            TokenPair {
                token_in: token_in.clone(),
                token_out: intermediary.clone(),
                fee_tier: fee_tier.clone(),
            },
            TokenPair {
                token_in: intermediary.clone(),
                token_out: token_out.clone(),
                fee_tier: fee_tier.clone(),
            },
        ]
    }
}

pub struct BasePair {
    pub native: EthereumAddress,
    pub stable: EthereumAddress,
}

impl BasePair {
    pub fn to_set(&self) -> HashSet<EthereumAddress> {
        HashSet::from([self.native.clone(), self.stable.clone()])
    }
}

impl From<AssetId> for EthereumAddress {
    fn from(asset_id: AssetId) -> Self {
        let token_id = asset_id.token_id.unwrap_or_default();
        EthereumAddress::parse(&token_id).unwrap()
    }
}

pub fn get_base_pair(chain: &EVMChain) -> Option<BasePair> {
    let weth = chain.weth_contract()?;
    let stable = match chain {
        EVMChain::Ethereum => "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        EVMChain::Polygon => "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359",
        EVMChain::Arbitrum => "0xaf88d065e77c8cc2239327c5edb3a432268e5831",
        EVMChain::Optimism => "0x0b2c639c533813f4aa9d7837caf62653d097ff85",
        EVMChain::Base => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        EVMChain::AvalancheC => "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
        EVMChain::Celo => "0xcebA9300f2b948710d2653dD7B07f33A8B32118C",
        EVMChain::SmartChain => "0x55d398326f99059fF775485246999027B3197955", // USDT (binance pegged)
        EVMChain::ZkSync => "0x1d17cbcf0d6d143135ae902365d2e5e2a16538d4",
        EVMChain::Blast => "0x4300000000000000000000000000000000000003", // USDB
        EVMChain::World => "0x79a02482a880bce3f13e09da970dc34db4cd24d1", // USDC.e
        _ => panic!("unsupported chain"),
    };

    let native = EthereumAddress::parse(weth)?;
    let stable = EthereumAddress::parse(stable)?;
    Some(BasePair { native, stable })
}

pub fn build_direct_pair(token_in: &EthereumAddress, token_out: &EthereumAddress, fee_tier: u32) -> Bytes {
    let mut bytes: Vec<u8> = vec![];
    let fee = U24::from(fee_tier);
    bytes.extend(&token_in.bytes);
    bytes.extend(&fee.to_be_bytes_vec());
    bytes.extend(&token_out.bytes);
    Bytes::from(bytes)
}

pub fn validate_pairs(token_pairs: &[TokenPair]) -> bool {
    // verify token in and out are chained
    let mut iter = token_pairs.iter().peekable();
    let mut valid = true;
    while let Some(current_pair) = iter.next() {
        if let Some(next_pair) = iter.peek() {
            if current_pair.token_out != next_pair.token_in {
                valid = false;
                break;
            }
        }
    }
    valid
}

pub fn build_pairs(token_pairs: &[TokenPair]) -> Bytes {
    let valid = validate_pairs(token_pairs);
    if !valid {
        panic!("invalid token pairs");
    }

    let mut bytes: Vec<u8> = vec![];
    for (idx, token_pair) in token_pairs.iter().enumerate() {
        let fee = U24::from::<u32>(token_pair.fee_tier.clone() as u32);
        if idx == 0 {
            bytes.extend(&token_pair.token_in.bytes);
        }
        bytes.extend(&fee.to_be_bytes_vec());
        bytes.extend(&token_pair.token_out.bytes);
    }
    Bytes::from(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_core::primitives::hex::encode_prefixed as HexEncode;

    #[test]
    fn test_build_path() {
        // Optimism WETH
        let token0 = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        // USDC
        let token1 = EthereumAddress::parse("0x0b2c639c533813f4aa9d7837caf62653d097ff85").unwrap();
        let bytes = build_direct_pair(&token0, &token1, FeeTier::FiveHundred as u32);

        assert_eq!(
            HexEncode(bytes),
            "0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85"
        )
    }

    #[test]
    fn test_two_hop_path() {
        // UNI
        let token0 = EthereumAddress::parse("0x6fd9d7AD17242c41f7131d257212c54A0e816691").unwrap();
        // WETH
        let token1 = EthereumAddress::parse("0x4200000000000000000000000000000000000006").unwrap();
        // LINK
        let token2 = EthereumAddress::parse("0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6").unwrap();
        let token_pairs = TokenPair::new_two_hop(&token0, &token1, &token2, &FeeTier::ThreeThousand);
        let bytes = build_pairs(&token_pairs);

        assert_eq!(
            HexEncode(bytes),
            "0x6fd9d7ad17242c41f7131d257212c54a0e816691000bb84200000000000000000000000000000000000006000bb8350a791bfc2c21f9ed5d10980dad2e2638ffa7f6"
        )
    }
}
