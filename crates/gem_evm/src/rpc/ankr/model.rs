use num_bigint::BigUint;
use primitives::EVMChain;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_hex_str;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub assets: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub contract_address: Option<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub balance_raw_integer: BigUint,
}

pub fn ankr_chain(chain: EVMChain) -> Option<String> {
    match chain {
        EVMChain::Ethereum => Some("eth".to_string()),
        EVMChain::Polygon => Some("polygon".to_string()),
        EVMChain::AvalancheC => Some("avalanche".to_string()),
        EVMChain::SmartChain => Some("bsc".to_string()),
        EVMChain::Arbitrum => Some("arbitrum".to_string()),
        EVMChain::Optimism => Some("optimism".to_string()),
        EVMChain::Base => Some("base".to_string()),
        EVMChain::OpBNB => None,
        EVMChain::Fantom => Some("fantom".to_string()),
        EVMChain::Gnosis => Some("gnosis".to_string()),
        EVMChain::Manta => None,
        EVMChain::Blast => None,  //Some("blast".to_string()),
        EVMChain::ZkSync => None, //Some("zksync_era".to_string()),
        EVMChain::Linea => Some("linea".to_string()),
        EVMChain::Mantle => None, //Some("mantle".to_string()),
        EVMChain::Celo => None,   //Some("celo".to_string()),
        EVMChain::World => None,
        EVMChain::Sonic => None, //Some("sonic_mainnet".to_string()),
        EVMChain::Abstract => None,
        EVMChain::Berachain => None,
        EVMChain::Ink => None,
        EVMChain::Unichain => None,
        EVMChain::Hyperliquid => None,
        EVMChain::Plasma => None,
        EVMChain::Monad => None,
    }
}
