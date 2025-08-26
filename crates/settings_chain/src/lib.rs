mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
use gem_client::{retry::standard_retry_policy, ReqwestClient};
pub use provider_config::ProviderConfig;

use reqwest_middleware::ClientBuilder;

use gem_chain_rpc::{
    ethereum::EthereumProvider, solana::SolanaProvider, ton::TonProvider, tron::TronProvider, ChainProvider, GenericProvider, HyperCoreProvider,
};

use gem_algorand::AlgorandClientIndexer;
use gem_aptos::rpc::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::{ankr::AnkrClient, AlchemyClient, EthereumClient};
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::PolkadotClient;
use gem_solana::rpc::client::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::SuiClient;
use gem_ton::rpc::TonClient;
use gem_tron::rpc::client::TronClient;
use gem_xrp::rpc::XRPClient;

use primitives::{chain_cosmos::CosmosChain, Chain, EVMChain, NodeType};
use settings::{ChainURLType, Settings};

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
        let url_type = Self::url(chain, settings);
        let url = url_type.get_url();
        let node_type = ProviderFactory::get_node_type(url_type.clone());
        Self::new_provider(ProviderConfig::new(
            chain,
            &url,
            node_type,
            settings.alchemy.key.secret.as_str(),
            settings.ankr.key.secret.as_str(),
            settings.trongrid.key.secret.as_str(),
        ))
    }

    pub fn new_providers(settings: &Settings) -> Vec<Box<dyn ChainProvider>> {
        Chain::all().iter().map(|x| Self::new_from_settings(*x, &settings.clone())).collect()
    }

    pub fn new_provider(config: ProviderConfig) -> Box<dyn ChainProvider> {
        let host = config
            .url
            .parse::<url::Url>()
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_default();

        let retry_policy = standard_retry_policy(host);

        let reqwest_client = reqwest::Client::builder().retry(retry_policy).build().expect("Failed to build reqwest client");

        let client = ClientBuilder::new(reqwest_client.clone()).build();
        let chain = config.chain;
        let url = config.url;
        let gem_client = ReqwestClient::new(url.clone(), reqwest_client.clone());

        match chain {
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge => Box::new(GenericProvider::new(BitcoinClient::new(
                gem_client,
                primitives::BitcoinChain::from_chain(chain).unwrap(),
            ))),
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
                let ethereum_client = EthereumClient::new(chain, &url);
                let assets_provider = AlchemyClient::new(ethereum_client.clone(), client.clone(), config.alchemy_key.clone());
                let transactions_provider = AnkrClient::new(ethereum_client.clone(), config.ankr_key.clone());
                Box::new(EthereumProvider::new(
                    ethereum_client,
                    config.node_type,
                    Box::new(assets_provider.clone()),
                    Box::new(transactions_provider.clone()),
                ))
            }
            Chain::Cardano => Box::new(GenericProvider::new(CardanoClient::new(gem_client))),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Noble | Chain::Sei => {
                let chain = CosmosChain::from_chain(chain).unwrap();
                Box::new(GenericProvider::new(CosmosClient::new(chain, gem_client.clone())))
            }
            Chain::Aptos => Box::new(GenericProvider::new(AptosClient::new(gem_client.clone()))),
            Chain::Sui => Box::new(GenericProvider::new(SuiClient::new(JsonRpcClient::new(gem_client.clone())))),
            Chain::Xrp => Box::new(GenericProvider::new(XRPClient::new(gem_client.clone()))),
            Chain::Algorand => Box::new(GenericProvider::new(AlgorandClientIndexer::new(gem_client.clone()))),
            Chain::Stellar => Box::new(GenericProvider::new(StellarClient::new(gem_client.clone()))),
            Chain::Near => Box::new(GenericProvider::new(NearClient::new(JsonRpcClient::new(gem_client.clone())))),
            Chain::Polkadot => Box::new(GenericProvider::new(PolkadotClient::new(gem_client.clone()))),
            Chain::Solana => Box::new(SolanaProvider::<ReqwestClient>::new(SolanaClient::new(JsonRpcClient::new(gem_client.clone())))),
            Chain::Ton => Box::new(TonProvider::new(TonClient::new(gem_client.clone()))),
            Chain::Tron => Box::new(TronProvider::new(TronClient::new(gem_client.clone()))),
            Chain::HyperCore => Box::new(HyperCoreProvider::new(HyperCoreClient::new(gem_client))),
        }
    }

    pub fn url(chain: Chain, settings: &Settings) -> ChainURLType {
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
