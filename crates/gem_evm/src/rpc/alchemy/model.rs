use num_bigint::BigUint;
use primitives::EVMChain;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_hex_str;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transactions {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub tokens: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub address: String,
    pub token_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub token_balance: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub hash: String,
}

pub fn evm_chain_to_network(chain: EVMChain) -> Option<String> {
    match chain {
        EVMChain::Ethereum => Some("eth-mainnet".to_string()),
        EVMChain::Base => Some("base-mainnet".to_string()),
        EVMChain::Arbitrum => Some("arb-mainnet".to_string()),
        EVMChain::Optimism => Some("opt-mainnet".to_string()),
        EVMChain::Polygon => Some("polygon-mainnet".to_string()),
        EVMChain::SmartChain => Some("bnb-mainnet".to_string()),
        EVMChain::AvalancheC => None, //Some("avalanche-mainnet".to_string()),
        EVMChain::OpBNB => Some("opbnb-mainnet".to_string()),
        EVMChain::Fantom => Some("fantom-mainnet".to_string()),
        EVMChain::Gnosis => Some("gnosis-mainnet".to_string()),
        EVMChain::Blast => Some("blast-mainnet".to_string()),
        EVMChain::ZkSync => Some("zksync-mainnet".to_string()),
        EVMChain::Linea => Some("linea-mainnet".to_string()),
        EVMChain::Mantle => Some("mantle-mainnet".to_string()),
        EVMChain::Celo => Some("celo-mainnet".to_string()),
        EVMChain::World => Some("worldchain-mainnet".to_string()),
        EVMChain::Sonic => Some("sonic-mainnet".to_string()),
        EVMChain::Abstract => Some("abstract-mainnet".to_string()),
        EVMChain::Berachain => Some("berachain-mainnet".to_string()),
        EVMChain::Ink => Some("ink-mainnet".to_string()),
        EVMChain::Unichain => Some("unichain-mainnet".to_string()),
        EVMChain::Manta => Some("manta-mainnet".to_string()),
        EVMChain::Hyperliquid => Some("hyperliquid-mainnet".to_string()),
        EVMChain::Monad => Some("monad-mainnet".to_string()),
    }
}
