use alloy_primitives::{Address, Bytes, aliases::U24};
use std::fmt::Display;

use super::FeeTier;
use primitives::{EVMChain, asset_constants::*};

#[derive(Debug, Clone, PartialEq)]
pub struct TokenPair {
    pub token_in: Address,
    pub token_out: Address,
    pub fee_tier: FeeTier,
}

impl Display for TokenPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}->{}", self.token_in, self.fee_tier as u32, self.token_out)
    }
}

#[derive(Debug, Clone)]
pub struct TokenPairs(pub Vec<TokenPair>);

impl Display for TokenPairs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?; // Write first element without a leading comma
            for item in iter {
                write!(f, ", {item}")?; // Write subsequent elements with a leading comma
            }
        }
        write!(f, "]")
    }
}

impl TokenPair {
    pub fn new_two_hop(token_in: &Address, intermediary: &Address, token_out: &Address, fee_tier: FeeTier) -> Vec<TokenPair> {
        vec![
            TokenPair {
                token_in: *token_in,
                token_out: *intermediary,
                fee_tier,
            },
            TokenPair {
                token_in: *intermediary,
                token_out: *token_out,
                fee_tier,
            },
        ]
    }
}

#[derive(Debug)]
pub struct BasePair {
    pub native: Address,
    pub stables: Vec<Address>,
    pub alternatives: Vec<Address>,
}

impl BasePair {
    pub fn path_building_array(&self) -> Vec<Address> {
        let mut array = vec![self.native];
        array.extend(self.stables.iter().cloned());
        // alternatives is not used for path building to reduce requests
        array
    }

    pub fn fee_token_array(&self) -> Vec<Address> {
        let mut array = vec![self.native];
        array.extend(self.stables.iter().cloned());
        array.extend(self.alternatives.iter().cloned());
        array
    }
}

pub fn get_base_pair(chain: &EVMChain, weth_as_native: bool) -> Option<BasePair> {
    let native = if weth_as_native { chain.weth_contract()?.parse().ok()? } else { Address::ZERO };

    let btc: &str = match chain {
        EVMChain::Ethereum => ETHEREUM_WBTC_TOKEN_ID,
        EVMChain::Polygon => POLYGON_WBTC_TOKEN_ID,
        EVMChain::Arbitrum => ARBITRUM_WBTC_TOKEN_ID,
        EVMChain::Optimism => OPTIMISM_WBTC_TOKEN_ID,
        EVMChain::Base => BASE_WBTC_TOKEN_ID,
        EVMChain::AvalancheC => "0x408d4cd0adb7cebd1f1a1c33a0ba2098e1295bab",
        EVMChain::Celo => "0xd71ffd0940c920786ec4dbb5a12306669b5b81ef",
        EVMChain::SmartChain => "0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c", // BTCB
        EVMChain::OpBNB => "0x7c6b91d9be155a6db01f749217d76ff02a7227f2",      // BTCB
        EVMChain::ZkSync => ZKSYNC_WBTC_TOKEN_ID,
        EVMChain::Blast => BLAST_WBTC_TOKEN_ID,
        EVMChain::World => WORLD_WBTC_TOKEN_ID,
        EVMChain::Sonic => SONIC_WBTC_TOKEN_ID,
        EVMChain::Linea => LINEA_WBTC_TOKEN_ID,
        EVMChain::SeiEvm => "0x0555E30da8f98308EdB960aa94C0Db47230d2B9c",
        _ => "", // None
    };

    let usdc = match chain {
        EVMChain::Ethereum => ETHEREUM_USDC_TOKEN_ID,
        EVMChain::Polygon => POLYGON_USDC_TOKEN_ID,
        EVMChain::Arbitrum => ARBITRUM_USDC_TOKEN_ID,
        EVMChain::Optimism => OPTIMISM_USDC_TOKEN_ID,
        EVMChain::Base => BASE_USDC_TOKEN_ID,
        EVMChain::AvalancheC => AVALANCHE_USDC_TOKEN_ID,
        EVMChain::Celo => "0xcebA9300f2b948710d2653dD7B07f33A8B32118C",
        EVMChain::SmartChain => SMARTCHAIN_USDC_TOKEN_ID,
        EVMChain::ZkSync => ZKSYNC_USDC_E_TOKEN_ID,
        EVMChain::Blast => "0x4300000000000000000000000000000000000003", // USDB
        EVMChain::World => WORLD_USDC_E_TOKEN_ID,
        EVMChain::Abstract => "0x84A71ccD554Cc1b02749b35d22F684CC8ec987e1", // USDC.e
        EVMChain::Unichain => UNICHAIN_USDC_TOKEN_ID,
        EVMChain::Sonic => "0x29219dd400f2bf60e5a23d13be72b486d4038894", // USDC.e
        EVMChain::Mantle => "0x09Bc4E0D864854c6aFB6eB9A9cdF58aC190D0dF9",
        EVMChain::Gnosis => GNOSIS_USDC_TOKEN_ID,
        EVMChain::Manta => "0xb73603c5d87fa094b7314c74ace2e64d165016fb",
        EVMChain::Linea => LINEA_USDC_E_TOKEN_ID,
        EVMChain::Ink => "0xF1815bd50389c46847f0Bda824eC8da914045D14",
        EVMChain::Monad => MONAD_USDC_TOKEN_ID,
        EVMChain::SeiEvm => "0xe15fC38F6D8c56aF07bbCBe3BAf5708A2Bf42392", // USDC (Circle native)
        EVMChain::OpBNB | EVMChain::Plasma => "",
        EVMChain::Stable => "0x8a2b28364102bea189d99a475c494330ef2bdd0b", // USDC.e (Stargate)
        _ => panic!("USDC is not configured for this chain"),
    };

    let usdt: &str = match chain {
        EVMChain::Ethereum => ETHEREUM_USDT_TOKEN_ID,
        EVMChain::Polygon => POLYGON_USDT_TOKEN_ID,
        EVMChain::Arbitrum => ARBITRUM_USDT_TOKEN_ID,
        EVMChain::Optimism => OPTIMISM_USDT_TOKEN_ID,
        EVMChain::Base => "0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2",
        EVMChain::AvalancheC => AVALANCHE_USDT_TOKEN_ID,
        EVMChain::Celo => "0x48065fbBE25f71C9282ddf5e1cD6D6A887483D5e",
        EVMChain::SmartChain => SMARTCHAIN_USDT_TOKEN_ID,
        EVMChain::ZkSync => ZKSYNC_USDT_TOKEN_ID,
        EVMChain::Abstract => "0x0709F39376dEEe2A2dfC94A58EdEb2Eb9DF012bD",
        EVMChain::Unichain => "0x9151434b16b9763660705744891fA906F660EcC5", // USDT0
        EVMChain::Sonic => "0x6047828dc181963ba44974801FF68e538dA5eaF9",
        EVMChain::Mantle => "0x201EBa5CC46D216Ce6DC03F6a759e8E766e956aE",
        EVMChain::Gnosis => GNOSIS_USDT_TOKEN_ID,
        EVMChain::Manta => "0xf417f5a458ec102b90352f697d6e2ac3a3d2851f",
        EVMChain::Linea => LINEA_USDT_TOKEN_ID,
        EVMChain::OpBNB => "0x9e5AAC1Ba1a2e6aEd6b32689DFcF62A509Ca96f3",
        EVMChain::Ink => INK_USDT_TOKEN_ID,
        EVMChain::Plasma => PLASMA_USDT_TOKEN_ID,
        EVMChain::Monad => MONAD_USDT_TOKEN_ID,
        EVMChain::SeiEvm => "0x9151434b16b9763660705744891fA906F660EcC5", // USDT0
        EVMChain::Stable => "0x779Ded0c9e1022225f8E0630b35a9b54bE713736", // USDT0
        EVMChain::Blast | EVMChain::World => "",                          // None
        _ => panic!("USDT is not configured for this chain"),
    };

    let mut stables = vec![];
    if !usdc.is_empty() {
        stables.push(usdc.parse().ok()?);
    }
    if !usdt.is_empty() {
        stables.push(usdt.parse().ok()?);
    }
    let alternatives = { if btc.is_empty() { vec![] } else { vec![btc.parse().ok()?] } };

    Some(BasePair { native, stables, alternatives })
}

pub fn build_direct_pair(token_in: &Address, token_out: &Address, fee_tier: FeeTier) -> Bytes {
    let mut bytes: Vec<u8> = vec![];
    let fee = U24::from(fee_tier.as_u24());
    bytes.extend(token_in.as_slice());
    bytes.extend(&fee.to_be_bytes_vec());
    bytes.extend(token_out.as_slice());
    Bytes::from(bytes)
}

pub fn validate_pairs(token_pairs: &[TokenPair]) -> bool {
    // verify token in and out are chained
    let mut iter = token_pairs.iter().peekable();
    let mut valid = true;
    while let Some(current_pair) = iter.next() {
        if let Some(next_pair) = iter.peek()
            && current_pair.token_out != next_pair.token_in
        {
            valid = false;
            break;
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
        let fee = U24::from(token_pair.fee_tier.as_u24());
        if idx == 0 {
            bytes.extend(token_pair.token_in.as_slice());
        }
        bytes.extend(&fee.to_be_bytes_vec());
        bytes.extend(token_pair.token_out.as_slice());
    }
    Bytes::from(bytes)
}

pub fn decode_path(path: &Bytes) -> Option<TokenPair> {
    // Minimum path: token_in | fee | token_out. Length = 20 + 3 + 20 = 43 bytes.
    if path.len() < 43 {
        return None;
    }

    let token_in = Address::from_slice(&path[0..20]);

    // Fee is a uint24, stored in 3 bytes.
    let fee_value = u32::from_be_bytes([0, path[20], path[21], path[22]]);
    let fee_tier = FeeTier::try_from(fee_value).ok()?;

    let token_out_offset = path.len() - 20;
    let token_out = Address::from_slice(&path[token_out_offset..path.len()]);

    Some(TokenPair { token_in, token_out, fee_tier })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, address, hex::encode_prefixed as HexEncode};
    use primitives::asset_constants::OPTIMISM_WETH_TOKEN_ID;

    #[test]
    fn test_build_path() {
        // Optimism WETH
        let token0: Address = OPTIMISM_WETH_TOKEN_ID.parse().unwrap();
        // USDC
        let token1: Address = OPTIMISM_USDC_TOKEN_ID.parse().unwrap();
        let bytes = build_direct_pair(&token0, &token1, FeeTier::FiveHundred);

        assert_eq!(
            HexEncode(&bytes),
            "0x42000000000000000000000000000000000000060001f40b2c639c533813f4aa9d7837caf62653d097ff85"
        );

        let pair = decode_path(&bytes).unwrap();
        assert_eq!(
            pair,
            TokenPair {
                token_in: token0,
                token_out: token1,
                fee_tier: FeeTier::FiveHundred
            }
        );
    }

    #[test]
    fn test_two_hop_path() {
        // UNI
        let token0 = address!("0x6fd9d7AD17242c41f7131d257212c54A0e816691");
        // WETH
        let token1: Address = OPTIMISM_WETH_TOKEN_ID.parse().unwrap();
        // LINK
        let token2 = address!("0x350a791Bfc2C21F9Ed5d10980Dad2e2638ffa7f6");
        let token_pairs = TokenPair::new_two_hop(&token0, &token1, &token2, FeeTier::ThreeThousand);
        let bytes = build_pairs(&token_pairs);

        assert_eq!(
            HexEncode(&bytes),
            "0x6fd9d7ad17242c41f7131d257212c54a0e816691000bb84200000000000000000000000000000000000006000bb8350a791bfc2c21f9ed5d10980dad2e2638ffa7f6"
        );

        let pair = decode_path(&bytes).unwrap();
        assert_eq!(
            pair,
            TokenPair {
                token_in: token0,
                token_out: token2,
                fee_tier: FeeTier::ThreeThousand
            }
        );
    }
}
