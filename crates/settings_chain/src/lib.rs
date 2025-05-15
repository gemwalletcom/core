use core::str;

use gem_chain_rpc::{
    algorand::client::AlgorandClient, algorand::provider::AlgorandProvider, aptos::client::AptosClient, aptos::provider::AptosProvider,
    bitcoin::client::BitcoinClient, bitcoin::provider::BitcoinProvider, cardano::client::CardanoClient, cardano::provider::CardanoProvider,
    cosmos::client::CosmosClient, cosmos::provider::CosmosProvider, ethereum::client::EthereumClient, ethereum::provider::EthereumProvider,
    near::client::NearClient, near::provider::NearProvider, polkadot::client::PolkadotClient, polkadot::provider::PolkadotProvider,
    solana::client::SolanaClient, solana::provider::SolanaProvider, stellar::client::StellarClient, stellar::provider::StellarProvider, sui::client::SuiClient,
    sui::provider::SuiProvider, ton::client::TonClient, ton::provider::TonProvider, tron::client::TronClient, tron::provider::TronProvider,
    xrp::client::XRPClient, xrp::provider::XRPProvider, ChainProvider,
};
use primitives::{Asset, Chain};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_from_settings(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
        let url = Self::url(chain, settings);
        Self::new_provider(chain, url)
    }

    pub fn new_providers(settings: &Settings) -> ChainProviders {
        let providers = Chain::all().iter().map(|x| Self::new_from_settings(*x, &settings.clone())).collect();
        ChainProviders::new(providers)
    }

    pub fn new_provider(chain: Chain, url: &str) -> Box<dyn ChainProvider> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let url = url.to_string();

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
            | Chain::Monad => Box::new(EthereumProvider::new(EthereumClient::new(chain, url))),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Noble | Chain::Sei => {
                Box::new(CosmosProvider::new(CosmosClient::new(chain, client, url)))
            }
            Chain::Solana => Box::new(SolanaProvider::new(SolanaClient::new(url.as_str()))),
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

pub struct ChainProviders {
    providers: Vec<Box<dyn ChainProvider>>,
}

impl ChainProviders {
    pub fn new(providers: Vec<Box<dyn ChainProvider>>) -> Self {
        Self { providers }
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        self.providers.iter().find(|x| x.get_chain() == chain).unwrap().get_token_data(token_id).await
    }
}
