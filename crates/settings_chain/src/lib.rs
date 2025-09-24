mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
use gem_algorand::{AlgorandClient, rpc::AlgorandClientIndexer};
use gem_client::{ReqwestClient, retry_policy};
use gem_hypercore::rpc::client::HyperCoreClient;
pub use provider_config::ProviderConfig;

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
use settings::{ChainURLType, Settings};

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainTraits> {
        let (url_type, archive_url_type) = Self::url(chain, settings);
        let url = url_type.get_url().unwrap_or_default();
        let archive_url = archive_url_type.unwrap_or(url_type.clone()).get_url().unwrap_or_default();
        Self::new_provider(ProviderConfig::new(
            chain,
            &url,
            &archive_url,
            settings.ankr.key.secret.as_str(),
            settings.trongrid.key.secret.as_str(),
        ))
    }

    pub fn new_providers(settings: &Settings) -> Vec<Box<dyn ChainTraits>> {
        Chain::all().iter().map(|x| Self::new_from_settings(*x, &settings.clone())).collect()
    }

    pub fn new_provider(config: ProviderConfig) -> Box<dyn ChainTraits> {
        let host = config
            .url
            .parse::<url::Url>()
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_default();

        let retry_policy_config = retry_policy(host, 3);
        let reqwest_client = reqwest::Client::builder()
            .retry(retry_policy_config)
            .build()
            .expect("Failed to build reqwest client");

        let chain = config.chain;
        let url = config.url.clone();
        let archive_url = config.archive_url.clone();
        let gem_client = ReqwestClient::new(url.clone(), reqwest_client.clone());
        let gem_client_archive = ReqwestClient::new(archive_url.clone(), reqwest_client.clone());

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
            | Chain::Sonic
            | Chain::Abstract
            | Chain::Berachain
            | Chain::Ink
            | Chain::Unichain
            | Chain::Hyperliquid
            | Chain::Monad => {
                let chain = EVMChain::from_chain(chain).unwrap();
                let node_type = config.clone().node_type();
                let client = if node_type == NodeType::Archive {
                    gem_client_archive.clone()
                } else {
                    gem_client.clone()
                };
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
            Chain::Xrp => Box::new(XRPClient::new(gem_client.clone())),
            Chain::Algorand => Box::new(AlgorandClient::new(gem_client.clone(), AlgorandClientIndexer::new(gem_client_archive.clone()))),
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

    pub fn url(chain: Chain, settings: &Settings) -> (ChainURLType, Option<ChainURLType>) {
        match chain {
            Chain::Bitcoin => settings.chains.bitcoin.get_type(),
            Chain::BitcoinCash => settings.chains.bitcoincash.get_type(),
            Chain::Litecoin => settings.chains.litecoin.get_type(),
            Chain::Ethereum => settings.chains.ethereum.get_type(),
            Chain::SmartChain => settings.chains.smartchain.get_type(),
            Chain::Solana => settings.chains.solana.get_type(),
            Chain::Polygon => settings.chains.polygon.get_type(),
            Chain::Thorchain => settings.chains.thorchain.get_type(),
            Chain::Cosmos => settings.chains.cosmos.get_type(),
            Chain::Osmosis => settings.chains.osmosis.get_type(),
            Chain::Arbitrum => settings.chains.arbitrum.get_type(),
            Chain::Ton => settings.chains.ton.get_type(),
            Chain::Tron => settings.chains.tron.get_type(),
            Chain::Doge => settings.chains.doge.get_type(),
            Chain::Optimism => settings.chains.optimism.get_type(),
            Chain::Aptos => settings.chains.aptos.get_type(),
            Chain::Base => settings.chains.base.get_type(),
            Chain::AvalancheC => settings.chains.avalanchec.get_type(),
            Chain::Sui => settings.chains.sui.get_type(),
            Chain::Xrp => settings.chains.xrp.get_type(),
            Chain::OpBNB => settings.chains.opbnb.get_type(),
            Chain::Fantom => settings.chains.fantom.get_type(),
            Chain::Gnosis => settings.chains.gnosis.get_type(),
            Chain::Celestia => settings.chains.celestia.get_type(),
            Chain::Injective => settings.chains.injective.get_type(),
            Chain::Sei => settings.chains.sei.get_type(),
            Chain::Manta => settings.chains.manta.get_type(),
            Chain::Blast => settings.chains.blast.get_type(),
            Chain::Noble => settings.chains.noble.get_type(),
            Chain::ZkSync => settings.chains.zksync.get_type(),
            Chain::Linea => settings.chains.linea.get_type(),
            Chain::Mantle => settings.chains.mantle.get_type(),
            Chain::Celo => settings.chains.celo.get_type(),
            Chain::Near => settings.chains.near.get_type(),
            Chain::World => settings.chains.world.get_type(),
            Chain::Stellar => settings.chains.stellar.get_type(),
            Chain::Sonic => settings.chains.sonic.get_type(),
            Chain::Algorand => settings.chains.algorand.get_type(),
            Chain::Polkadot => settings.chains.polkadot.get_type(),
            Chain::Cardano => settings.chains.cardano.get_type(),
            Chain::Abstract => settings.chains.abstract_chain.get_type(),
            Chain::Berachain => settings.chains.berachain.get_type(),
            Chain::Ink => settings.chains.ink.get_type(),
            Chain::Unichain => settings.chains.unichain.get_type(),
            Chain::Hyperliquid => settings.chains.hyperliquid.get_type(),
            Chain::HyperCore => settings.chains.hypercore.get_type(),
            Chain::Monad => settings.chains.monad.get_type(),
        }
    }

    pub fn get_node_type(url: ChainURLType) -> NodeType {
        match url {
            ChainURLType::Default(_) => NodeType::Default,
            ChainURLType::Archive(_) => NodeType::Archive,
        }
    }
}
