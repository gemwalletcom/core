mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
use gem_jsonrpc::JsonRpcClient;
pub use provider_config::ProviderConfig;

use core::str;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};

use gem_chain_rpc::{
    algorand::AlgorandProvider, bitcoin::BitcoinProvider, cardano::CardanoProvider, ethereum::EthereumProvider, solana::SolanaProvider, sui::SuiProvider,
    ton::TonProvider, tron::TronProvider, xrp::XRPProvider, AptosProvider, ChainProvider, CosmosProvider, NearProvider, PolkadotProvider, StellarProvider,
};

use gem_algorand::rpc::AlgorandClient;
use gem_aptos::rpc::AptosClient;
use gem_bitcoin::rpc::BitcoinClient;
use gem_cardano::rpc::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::{ankr::AnkrClient, AlchemyClient, EthereumClient};
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::PolkadotClient;
use gem_solana::rpc::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::SuiClient;
use gem_ton::rpc::TonClient;
use gem_tron::rpc::{trongrid::client::TronGridClient, TronClient};
use gem_xrp::rpc::XRPClient;

use primitives::{chain_cosmos::CosmosChain, Chain, EVMChain};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
        let url = Self::url(chain, settings);
        Self::new_provider(ProviderConfig::new(
            chain,
            url,
            settings.alchemy.key.secret.as_str(),
            settings.ankr.key.secret.as_str(),
            settings.trongrid.key.secret.as_str(),
        ))
    }

    pub fn new_providers(settings: &Settings) -> Vec<Box<dyn ChainProvider>> {
        Chain::all().iter().map(|x| Self::new_from_settings(*x, &settings.clone())).collect()
    }

    pub fn new_provider(config: ProviderConfig) -> Box<dyn ChainProvider> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let chain = config.chain;
        let url = config.url;

        match chain {
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge => Box::new(BitcoinProvider::new(BitcoinClient::new(chain, client, url))),
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
                    Box::new(assets_provider.clone()),
                    Box::new(transactions_provider.clone()),
                ))
            }
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Noble | Chain::Sei => {
                let chain = CosmosChain::from_chain(chain).unwrap();
                Box::new(CosmosProvider::new(CosmosClient::new(chain, client, url)))
            }
            Chain::Solana => Box::new(SolanaProvider::new(SolanaClient::new(&url))),
            Chain::Ton => Box::new(TonProvider::new(TonClient::new(client, url))),
            Chain::Tron => {
                let client = TronClient::new(client, url.clone());
                let grid_client = TronGridClient::new(client.clone(), url.clone(), config.trongrid_key.clone());
                Box::new(TronProvider::new(client, Box::new(grid_client.clone()), Box::new(grid_client.clone())))
            }
            Chain::Aptos => Box::new(AptosProvider::new(AptosClient::new(client, url))),
            Chain::Sui => Box::new(SuiProvider::new(SuiClient::new(JsonRpcClient::new_with_client(url, client)))),
            Chain::Xrp => Box::new(XRPProvider::new(XRPClient::new(client, url))),
            Chain::Near => Box::new(NearProvider::new(NearClient::new(JsonRpcClient::new_with_client(url, client)))),
            Chain::Cardano => Box::new(CardanoProvider::new(CardanoClient::new(client, url))),
            Chain::Algorand => Box::new(AlgorandProvider::new(AlgorandClient::new(client, url))),
            Chain::Stellar => Box::new(StellarProvider::new(StellarClient::new(client, url))),
            Chain::Polkadot => Box::new(PolkadotProvider::new(PolkadotClient::new(client, url))),
        }
    }

    pub fn url(chain: Chain, settings: &Settings) -> &str {
        match chain {
            Chain::Bitcoin => settings.chains.bitcoin.get_url(),
            Chain::BitcoinCash => settings.chains.bitcoincash.get_url(),
            Chain::Litecoin => settings.chains.litecoin.get_url(),
            Chain::Ethereum => settings.chains.ethereum.get_url(),
            Chain::SmartChain => settings.chains.smartchain.get_url(),
            Chain::Solana => settings.chains.solana.get_url(),
            Chain::Polygon => settings.chains.polygon.get_url(),
            Chain::Thorchain => settings.chains.thorchain.get_url(),
            Chain::Cosmos => settings.chains.cosmos.get_url(),
            Chain::Osmosis => settings.chains.osmosis.get_url(),
            Chain::Arbitrum => settings.chains.arbitrum.get_url(),
            Chain::Ton => settings.chains.ton.get_url(),
            Chain::Tron => settings.chains.tron.get_url(),
            Chain::Doge => settings.chains.doge.get_url(),
            Chain::Optimism => settings.chains.optimism.get_url(),
            Chain::Aptos => settings.chains.aptos.get_url(),
            Chain::Base => settings.chains.base.get_url(),
            Chain::AvalancheC => settings.chains.avalanchec.get_url(),
            Chain::Sui => settings.chains.sui.get_url(),
            Chain::Xrp => settings.chains.xrp.get_url(),
            Chain::OpBNB => settings.chains.opbnb.get_url(),
            Chain::Fantom => settings.chains.fantom.get_url(),
            Chain::Gnosis => settings.chains.gnosis.get_url(),
            Chain::Celestia => settings.chains.celestia.get_url(),
            Chain::Injective => settings.chains.injective.get_url(),
            Chain::Sei => settings.chains.sei.get_url(),
            Chain::Manta => settings.chains.manta.get_url(),
            Chain::Blast => settings.chains.blast.get_url(),
            Chain::Noble => settings.chains.noble.get_url(),
            Chain::ZkSync => settings.chains.zksync.get_url(),
            Chain::Linea => settings.chains.linea.get_url(),
            Chain::Mantle => settings.chains.mantle.get_url(),
            Chain::Celo => settings.chains.celo.get_url(),
            Chain::Near => settings.chains.near.get_url(),
            Chain::World => settings.chains.world.get_url(),
            Chain::Stellar => settings.chains.stellar.get_url(),
            Chain::Sonic => settings.chains.sonic.get_url(),
            Chain::Algorand => settings.chains.algorand.get_url(),
            Chain::Polkadot => settings.chains.polkadot.get_url(),
            Chain::Cardano => settings.chains.cardano.get_url(),
            Chain::Abstract => settings.chains.abstract_chain.get_url(),
            Chain::Berachain => settings.chains.berachain.get_url(),
            Chain::Ink => settings.chains.ink.get_url(),
            Chain::Unichain => settings.chains.unichain.get_url(),
            Chain::Hyperliquid => settings.chains.hyperliquid.get_url(),
            Chain::Monad => settings.chains.monad.get_url(),
        }
    }
}
