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

pub fn evm_chain_to_network(chain: EVMChain) -> String {
    match chain {
        EVMChain::Ethereum => "eth-mainnet".to_string(),
        EVMChain::Base => "base-mainnet".to_string(),
        EVMChain::Arbitrum => "arb-mainnet".to_string(),
        EVMChain::Optimism => "opt-mainnet".to_string(),
        EVMChain::Polygon => "polygon-mainnet".to_string(),
        EVMChain::SmartChain => "bnb-mainnet".to_string(),
        EVMChain::AvalancheC => "avalanche-mainnet".to_string(),
        EVMChain::OpBNB => "opbnb-mainnet".to_string(),
        EVMChain::Fantom => "fantom-mainnet".to_string(),
        EVMChain::Gnosis => "gnosis-mainnet".to_string(),
        EVMChain::Blast => "blast-mainnet".to_string(),
        EVMChain::ZkSync => "zksync-mainnet".to_string(),
        EVMChain::Linea => "linea-mainnet".to_string(),
        EVMChain::Mantle => "mantle-mainnet".to_string(),
        EVMChain::Celo => "celo-mainnet".to_string(),
        EVMChain::World => "worldchain-mainnet".to_string(),
        EVMChain::Sonic => "sonic-mainnet".to_string(),
        EVMChain::Abstract => "abstract-mainnet".to_string(),
        EVMChain::Berachain => "berachain-mainnet".to_string(),
        EVMChain::Ink => "ink-mainnet".to_string(),
        EVMChain::Unichain => "unichain-mainnet".to_string(),
        EVMChain::Manta => "manta-mainnet".to_string(),
        EVMChain::Hyperliquid => "hyperliquid-mainnet".to_string(),
        EVMChain::Monad => "monad-mainnet".to_string(),
    }
}
