mod chain_providers;
mod provider_config;
pub use chain_providers::ChainProviders;
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
use gem_evm::rpc::{AlchemyClient, EthereumClient};
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::PolkadotClient;
use gem_solana::rpc::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::SuiClient;
use gem_ton::rpc::TonClient;
use gem_tron::rpc::TronClient;
use gem_xrp::rpc::XRPClient;

use primitives::{Chain, EVMChain};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
        let url = Self::url(chain, settings);
        Self::new_provider(ProviderConfig::new(chain, url, settings.alchemy.key.secret.as_str()))
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
                let assets_provider = AlchemyClient::new(chain, &config.alchemy_key);
                Box::new(EthereumProvider::new(EthereumClient::new(chain, url), Box::new(assets_provider)))
            }
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Noble | Chain::Sei => {
                Box::new(CosmosProvider::new(CosmosClient::new(chain, client, url)))
            }
            Chain::Solana => Box::new(SolanaProvider::new(SolanaClient::new(&url))),
            Chain::Ton => Box::new(TonProvider::new(TonClient::new(client, url))),
            Chain::Tron => Box::new(TronProvider::new(TronClient::new(client, url))),
            Chain::Aptos => Box::new(AptosProvider::new(AptosClient::new(client, url))),
            Chain::Sui => Box::new(SuiProvider::new(SuiClient::new(url))),
            Chain::Xrp => Box::new(XRPProvider::new(XRPClient::new(client, url))),
            Chain::Near => Box::new(NearProvider::new(NearClient::new(url))),
            Chain::Cardano => Box::new(CardanoProvider::new(CardanoClient::new(client, url))),
            Chain::Algorand => Box::new(AlgorandProvider::new(AlgorandClient::new(client, url))),
            Chain::Stellar => Box::new(StellarProvider::new(StellarClient::new(client, url))),
            Chain::Polkadot => Box::new(PolkadotProvider::new(PolkadotClient::new(client, url))),
        }
    }

    pub fn url(chain: Chain, settings: &Settings) -> &str {
        match chain {
            Chain::Bitcoin => settings.chains.bitcoin.url.as_str(),
            Chain::BitcoinCash => settings.chains.bitcoincash.url.as_str(),
            Chain::Litecoin => settings.chains.litecoin.url.as_str(),
            Chain::Ethereum => settings.chains.ethereum.url.as_str(),
            Chain::SmartChain => settings.chains.smartchain.url.as_str(),
            Chain::Solana => settings.chains.solana.url.as_str(),
            Chain::Polygon => settings.chains.polygon.url.as_str(),
            Chain::Thorchain => settings.chains.thorchain.url.as_str(),
            Chain::Cosmos => settings.chains.cosmos.url.as_str(),
            Chain::Osmosis => settings.chains.osmosis.url.as_str(),
            Chain::Arbitrum => settings.chains.arbitrum.url.as_str(),
            Chain::Ton => settings.chains.ton.url.as_str(),
            Chain::Tron => settings.chains.tron.url.as_str(),
            Chain::Doge => settings.chains.doge.url.as_str(),
            Chain::Optimism => settings.chains.optimism.url.as_str(),
            Chain::Aptos => settings.chains.aptos.url.as_str(),
            Chain::Base => settings.chains.base.url.as_str(),
            Chain::AvalancheC => settings.chains.avalanchec.url.as_str(),
            Chain::Sui => settings.chains.sui.url.as_str(),
            Chain::Xrp => settings.chains.xrp.url.as_str(),
            Chain::OpBNB => settings.chains.opbnb.url.as_str(),
            Chain::Fantom => settings.chains.fantom.url.as_str(),
            Chain::Gnosis => settings.chains.gnosis.url.as_str(),
            Chain::Celestia => settings.chains.celestia.url.as_str(),
            Chain::Injective => settings.chains.injective.url.as_str(),
            Chain::Sei => settings.chains.sei.url.as_str(),
            Chain::Manta => settings.chains.manta.url.as_str(),
            Chain::Blast => settings.chains.blast.url.as_str(),
            Chain::Noble => settings.chains.noble.url.as_str(),
            Chain::ZkSync => settings.chains.zksync.url.as_str(),
            Chain::Linea => settings.chains.linea.url.as_str(),
            Chain::Mantle => settings.chains.mantle.url.as_str(),
            Chain::Celo => settings.chains.celo.url.as_str(),
            Chain::Near => settings.chains.near.url.as_str(),
            Chain::World => settings.chains.world.url.as_str(),
            Chain::Stellar => settings.chains.stellar.url.as_str(),
            Chain::Sonic => settings.chains.sonic.url.as_str(),
            Chain::Algorand => settings.chains.algorand.url.as_str(),
            Chain::Polkadot => settings.chains.polkadot.url.as_str(),
            Chain::Cardano => settings.chains.cardano.url.as_str(),
            Chain::Abstract => settings.chains.abstract_chain.url.as_str(),
            Chain::Berachain => settings.chains.berachain.url.as_str(),
            Chain::Ink => settings.chains.ink.url.as_str(),
            Chain::Unichain => settings.chains.unichain.url.as_str(),
            Chain::Hyperliquid => settings.chains.hyperliquid.url.as_str(),
            Chain::Monad => settings.chains.monad.url.as_str(),
        }
    }
}
