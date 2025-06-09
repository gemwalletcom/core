use num_bigint::BigUint;
use primitives::EVMChain;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_hex_str;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub contract_address: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub token_balance: BigUint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalances {
    pub address: Option<String>,
    pub token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfers {
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub asset: String,
    pub token_id: Option<String>,
    pub category: String,
    pub raw_contract: TransferRawData,
    pub metadata: TransferMetadata,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRawData {
    pub value: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMetadata {
    pub block_timestamp: String,
}

pub fn alchemy_url(chain: EVMChain, api_key: &str) -> String {
    let prefix = match chain {
        EVMChain::Ethereum => "eth",
        EVMChain::Base => "base",
        EVMChain::Arbitrum => "arb",
        EVMChain::Optimism => "opt",
        EVMChain::Polygon => "polygon",
        EVMChain::SmartChain => "bnb",
        EVMChain::AvalancheC => "avalanche",
        EVMChain::OpBNB => "opbnb",
        EVMChain::Fantom => "fantom",
        EVMChain::Gnosis => "gnosis",
        EVMChain::Blast => "blast",
        EVMChain::ZkSync => "zksync",
        EVMChain::Linea => "linea",
        EVMChain::Mantle => "mantle",
        EVMChain::Celo => "celo",
        EVMChain::World => "worldchain",
        EVMChain::Sonic => "sonic",
        EVMChain::Abstract => "abstract",
        EVMChain::Berachain => "berachain",
        EVMChain::Ink => "ink",
        EVMChain::Unichain => "unichain",
        EVMChain::Manta => "manta",             // TODO: no support
        EVMChain::Hyperliquid => "hyperliquid", // TODO: no support
        EVMChain::Monad => "monad",             // TODO: no support
    };
    format!("https://{}-mainnet.g.alchemy.com/v2/{}", prefix, api_key)
}
