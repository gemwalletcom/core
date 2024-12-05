use std::collections::HashSet;

use super::FeeTier;
use crate::address::EthereumAddress;

use alloy_core::primitives::Bytes;
use alloy_primitives::aliases::U24;
use primitives::{AssetId, EVMChain};

#[derive(Debug, Clone)]
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
    pub stables: Vec<EthereumAddress>,
    pub alternatives: Vec<EthereumAddress>,
}

impl BasePair {
    pub fn to_set(&self) -> HashSet<EthereumAddress> {
        HashSet::from_iter(self.to_array())
    }

    pub fn to_array(&self) -> Vec<EthereumAddress> {
        let mut array = vec![self.native.clone()];
        array.extend(self.stables.iter().cloned());
        // alternatives is not used for now
        // array.extend(self.alternatives.iter().cloned());
        array
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

    let btc: &str = match chain {
        EVMChain::Ethereum => "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
        EVMChain::Polygon => "0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6",
        EVMChain::Arbitrum => "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f",
        EVMChain::Optimism => "0x68f180fcCe6836688e9084f035309E29Bf0A2095",
        EVMChain::Base => "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c",
        EVMChain::AvalancheC => "0x408d4cd0adb7cebd1f1a1c33a0ba2098e1295bab",
        EVMChain::Celo => "0xd71ffd0940c920786ec4dbb5a12306669b5b81ef",
        EVMChain::SmartChain => "0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c", // BTCB
        EVMChain::ZkSync => "0xBBeB516fb02a01611cBBE0453Fe3c580D7281011",
        EVMChain::Blast => "0xf7bc58b8d8f97adc129cfc4c9f45ce3c0e1d2692",
        EVMChain::World => "0x03C7054BCB39f7b2e5B2c7AcB37583e32D70Cfa3",
        _ => panic!("unsupported chain"),
    };

    let usdc = match chain {
        EVMChain::Ethereum => "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        EVMChain::Polygon => "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359",
        EVMChain::Arbitrum => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
        EVMChain::Optimism => "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85",
        EVMChain::Base => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        EVMChain::AvalancheC => "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
        EVMChain::Celo => "0xcebA9300f2b948710d2653dD7B07f33A8B32118C",
        EVMChain::SmartChain => "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d",
        EVMChain::ZkSync => "0x3355df6D4c9C3035724Fd0e3914dE96A5a83aaf4", // USDC.e
        EVMChain::Blast => "0x4300000000000000000000000000000000000003",  // USDB
        EVMChain::World => "0x79A02482A880bCE3F13e09Da970dC34db4CD24d1",  // USDC.e
        _ => panic!("unsupported chain"),
    };

    let usdt: &str = match chain {
        EVMChain::Ethereum => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        EVMChain::Polygon => "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
        EVMChain::Arbitrum => "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9",
        EVMChain::Optimism => "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58",
        EVMChain::Base => "0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2",
        EVMChain::AvalancheC => "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7",
        EVMChain::Celo => "0x48065fbBE25f71C9282ddf5e1cD6D6A887483D5e",
        EVMChain::SmartChain => "0x55d398326f99059fF775485246999027B3197955",
        EVMChain::ZkSync => "0x493257fD37EDB34451f62EDf8D2a0C418852bA4C",
        EVMChain::Blast => "", // None
        EVMChain::World => "", // None
        _ => panic!("unsupported chain"),
    };

    let mut stables = vec![EthereumAddress::parse(usdc)?];
    if !usdt.is_empty() {
        stables.push(EthereumAddress::parse(usdt)?);
    }

    Some(BasePair {
        native: EthereumAddress::parse(weth)?,
        stables,
        alternatives: vec![EthereumAddress::parse(btc)?],
    })
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
