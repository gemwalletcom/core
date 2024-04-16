use gem_chain_rpc::{
    AptosClient, BitcoinClient, ChainProvider, CosmosClient, EthereumClient, NearClient,
    SolanaClient, SuiClient, TonClient, TronClient, XRPClient,
};
use primitives::Chain;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use settings::Settings;

pub struct ProviderFactory {}

impl ProviderFactory {
    pub fn new_provider(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(5);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let url = Self::url(chain, settings).to_string();

        match chain {
            Chain::Bitcoin | Chain::Litecoin | Chain::Doge => {
                Box::new(BitcoinClient::new(chain, client, url))
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
            | Chain::Celo => Box::new(EthereumClient::new(chain, url)),
            Chain::Cosmos
            | Chain::Osmosis
            | Chain::Celestia
            | Chain::Thorchain
            | Chain::Injective
            | Chain::Noble
            | Chain::Sei
            | Chain::Dymension
            | Chain::Saga => Box::new(CosmosClient::new(chain, client, url)),
            Chain::Solana => Box::new(SolanaClient::new(url)),
            Chain::Ton => Box::new(TonClient::new(client, url)),
            Chain::Tron => Box::new(TronClient::new(client, url)),
            Chain::Aptos => Box::new(AptosClient::new(client, url)),
            Chain::Sui => Box::new(SuiClient::new(url)),
            Chain::Xrp => Box::new(XRPClient::new(client, url)),
            Chain::Near => Box::new(NearClient::new(url)),
        }
    }

    pub fn url(chain: Chain, settings: &Settings) -> &str {
        match chain {
            Chain::Bitcoin => settings.chains.bitcoin.url.as_str(),
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
            Chain::Dymension => settings.chains.dimension.url.as_str(),
            Chain::Saga => settings.chains.saga.url.as_str(),
        }
    }
}
