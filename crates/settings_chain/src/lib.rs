mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
use gem_algorand::{
    AlgorandClient,
    rpc::{AlgorandClientIndexer, client_indexer::ALGORAND_INDEXER_URL},
};
use gem_client::{ReqwestClient, retry_policy};
use gem_hypercore::rpc::client::HyperCoreClient;
pub use provider_config::ProviderConfig;
pub use settings::ChainURLType;

use chain_traits::ChainTraits;

use gem_aptos::rpc::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::{EthereumClient, ankr::AnkrClient};
use gem_jsonrpc::client::JsonRpcClient;
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::PolkadotClient;
use gem_solana::rpc::client::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::SuiClient;
use gem_ton::rpc::TonClient;
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use gem_xrp::rpc::XRPClient;

use primitives::{Chain, EVMChain, NodeType, chain_cosmos::CosmosChain};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainTraits> {
        Self::new_from_settings_with_user_agent(chain, settings, "")
    }

    pub fn new_from_settings_with_user_agent(chain: Chain, settings: &Settings, user_agent: &str) -> Box<dyn ChainTraits> {
        let chain_config = Self::get_chain_config(chain, settings);
        let node_type = Self::get_node_type(chain_config.node.clone());

        Self::new_provider(
            ProviderConfig::new(
                chain,
                &chain_config.url,
                node_type,
                settings.ankr.key.secret.as_str(),
                settings.trongrid.key.secret.as_str(),
            ),
            user_agent,
        )
    }

    pub fn new_providers(settings: &Settings) -> Vec<Box<dyn ChainTraits>> {
        Chain::all().iter().map(|x| Self::new_from_settings(*x, &settings.clone())).collect()
    }

    pub fn new_providers_with_user_agent(settings: &Settings, user_agent: &str) -> Vec<Box<dyn ChainTraits>> {
        Chain::all()
            .iter()
            .map(|x| Self::new_from_settings_with_user_agent(*x, &settings.clone(), user_agent))
            .collect()
    }

    pub fn new_provider(config: ProviderConfig, user_agent: &str) -> Box<dyn ChainTraits> {
        let host = config
            .url
            .parse::<url::Url>()
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_default();

        let retry_policy_config = retry_policy(host, 3);
        let reqwest_client = gem_client::default_client_builder()
            .retry(retry_policy_config)
            .build()
            .expect("Failed to build reqwest client");

        let chain = config.chain;
        let url = config.url.clone();
        let node_type = config.clone().node_type;
        let gem_client = ReqwestClient::new_with_user_agent(url.clone(), reqwest_client.clone(), user_agent.to_string());

        match chain {
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge => {
                Box::new(BitcoinClient::new(gem_client, primitives::BitcoinChain::from_chain(chain).unwrap()))
            }
            Chain::Ethereum
            | Chain::SmartChain
            | Chain::Polygon
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::Arbitrum
            | Chain::Optimism
            | Chain::Base
            | Chain::AvalancheC
            | Chain::OpBNB
            | Chain::Manta
            | Chain::Blast
            | Chain::ZkSync
            | Chain::Linea
            | Chain::Mantle
            | Chain::Celo
            | Chain::World
            | Chain::Plasma
            | Chain::Sonic
            | Chain::Abstract
            | Chain::Berachain
            | Chain::Ink
            | Chain::Unichain
            | Chain::Hyperliquid
            | Chain::Monad => {
                let chain = EVMChain::from_chain(chain).unwrap();
                let client = gem_client.clone();
                let rpc_client = JsonRpcClient::new(client.clone());
                let ethereum_client = EthereumClient::new(rpc_client.clone(), chain)
                    .with_node_type(node_type)
                    .with_ankr_client(AnkrClient::new(
                        JsonRpcClient::new(ReqwestClient::new(config.clone().ankr_url(), reqwest_client.clone())),
                        chain,
                    ));
                Box::new(ethereum_client)
            }
            Chain::Cardano => Box::new(CardanoClient::new(gem_client)),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Noble | Chain::Sei => {
                let chain = CosmosChain::from_chain(chain).unwrap();
                Box::new(CosmosClient::new(chain, gem_client.clone()))
            }
            Chain::Aptos => Box::new(AptosClient::new(gem_client.clone())),
            Chain::Sui => Box::new(SuiClient::new(JsonRpcClient::new(gem_client.clone()))),
            Chain::Xrp => Box::new(XRPClient::new(JsonRpcClient::new(gem_client.clone()))),
            Chain::Algorand => {
                let indexer_client = ReqwestClient::new(ALGORAND_INDEXER_URL.to_string(), reqwest_client.clone());
                Box::new(AlgorandClient::new(gem_client.clone(), AlgorandClientIndexer::new(indexer_client)))
            }
            Chain::Stellar => Box::new(StellarClient::new(gem_client.clone())),
            Chain::Near => Box::new(NearClient::new(JsonRpcClient::new(gem_client.clone()))),
            Chain::Polkadot => Box::new(PolkadotClient::new(gem_client.clone())),
            Chain::Solana => Box::new(SolanaClient::new(JsonRpcClient::new(gem_client.clone()))),
            Chain::Ton => Box::new(TonClient::new(gem_client.clone())),
            Chain::Tron => Box::new(TronClient::new(
                gem_client.clone(),
                TronGridClient::new(gem_client.clone(), config.trongrid_key.clone()),
            )),
            Chain::HyperCore => Box::new(HyperCoreClient::new(gem_client.clone())),
        }
    }

    pub fn get_chain_config(chain: Chain, settings: &Settings) -> &settings::Chain {
        match chain {
            Chain::Bitcoin => &settings.chains.bitcoin,
            Chain::BitcoinCash => &settings.chains.bitcoincash,
            Chain::Litecoin => &settings.chains.litecoin,
            Chain::Ethereum => &settings.chains.ethereum,
            Chain::SmartChain => &settings.chains.smartchain,
            Chain::Solana => &settings.chains.solana,
            Chain::Polygon => &settings.chains.polygon,
            Chain::Thorchain => &settings.chains.thorchain,
            Chain::Cosmos => &settings.chains.cosmos,
            Chain::Osmosis => &settings.chains.osmosis,
            Chain::Arbitrum => &settings.chains.arbitrum,
            Chain::Ton => &settings.chains.ton,
            Chain::Tron => &settings.chains.tron,
            Chain::Doge => &settings.chains.doge,
            Chain::Optimism => &settings.chains.optimism,
            Chain::Aptos => &settings.chains.aptos,
            Chain::Base => &settings.chains.base,
            Chain::AvalancheC => &settings.chains.avalanchec,
            Chain::Sui => &settings.chains.sui,
            Chain::Xrp => &settings.chains.xrp,
            Chain::OpBNB => &settings.chains.opbnb,
            Chain::Fantom => &settings.chains.fantom,
            Chain::Gnosis => &settings.chains.gnosis,
            Chain::Celestia => &settings.chains.celestia,
            Chain::Injective => &settings.chains.injective,
            Chain::Sei => &settings.chains.sei,
            Chain::Manta => &settings.chains.manta,
            Chain::Blast => &settings.chains.blast,
            Chain::Noble => &settings.chains.noble,
            Chain::ZkSync => &settings.chains.zksync,
            Chain::Linea => &settings.chains.linea,
            Chain::Mantle => &settings.chains.mantle,
            Chain::Celo => &settings.chains.celo,
            Chain::Near => &settings.chains.near,
            Chain::World => &settings.chains.world,
            Chain::Plasma => &settings.chains.plasma,
            Chain::Stellar => &settings.chains.stellar,
            Chain::Sonic => &settings.chains.sonic,
            Chain::Algorand => &settings.chains.algorand,
            Chain::Polkadot => &settings.chains.polkadot,
            Chain::Cardano => &settings.chains.cardano,
            Chain::Abstract => &settings.chains.abstract_chain,
            Chain::Berachain => &settings.chains.berachain,
            Chain::Ink => &settings.chains.ink,
            Chain::Unichain => &settings.chains.unichain,
            Chain::Hyperliquid => &settings.chains.hyperliquid,
            Chain::HyperCore => &settings.chains.hypercore,
            Chain::Monad => &settings.chains.monad,
        }
    }

    pub fn get_node_type(url_type: ChainURLType) -> NodeType {
        match url_type {
            ChainURLType::Default => NodeType::Default,
            ChainURLType::Archival => NodeType::Archival,
        }
    }
}
