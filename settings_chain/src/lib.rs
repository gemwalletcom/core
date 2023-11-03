use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use blockchain::{ChainProvider, BNBChainClient, SolanaClient, EthereumClient, TonClient, CosmosClient, TronClient, XRPClient, AptosClient, SuiClient, BitcoinClient};
use settings::Settings;
use primitives::Chain;

pub struct ProviderFactory {
    
}

impl ProviderFactory {
    pub fn new(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {

        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(5);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
    
        match chain {
            Chain::Bitcoin => Box::new(BitcoinClient::new(
                Chain::Bitcoin,
                client,
                settings.chains.bitcoin.url.clone()
            )),
            Chain::Ethereum => Box::new(EthereumClient::new(
                Chain::Ethereum,
                settings.chains.ethereum.url.clone()
            )),
            Chain::Binance => Box::new(BNBChainClient::new(
                client,
                settings.chains.binance.url.clone(),
                settings.chains.binance.api.clone(),
            )),
            Chain::SmartChain => Box::new(EthereumClient::new(
                Chain::SmartChain,
                settings.chains.smartchain.url.clone()
            )),
            Chain::Solana => Box::new(SolanaClient::new(settings.chains.solana.url.clone())),
            Chain::Polygon => Box::new(EthereumClient::new(
                Chain::Polygon,
                settings.chains.polygon.url.clone()
            )),
            Chain::Thorchain => Box::new(CosmosClient::new(
                Chain::Thorchain,
                client,
                settings.chains.thorchain.url.clone()
            )),
            Chain::Cosmos => Box::new(CosmosClient::new(
                Chain::Cosmos,
                client,
                settings.chains.cosmos.url.clone()
            )),
            Chain::Osmosis => Box::new(CosmosClient::new(
                Chain::Osmosis,
                client,
                settings.chains.osmosis.url.clone()
            )),
            Chain::Arbitrum => Box::new(EthereumClient::new(
                Chain::Arbitrum,
                settings.chains.arbitrum.url.clone()
            )),
            Chain::Ton => Box::new(TonClient::new(
                client,
                settings.chains.ton.url.clone()
            )),
            Chain::Tron => Box::new(TronClient::new(
                client,
                settings.chains.tron.url.clone()
            )),
            Chain::Doge => Box::new(BitcoinClient::new(
                Chain::Doge,
                client,
                settings.chains.doge.url.clone()
            )),
            Chain::Optimism => Box::new(EthereumClient::new(
                Chain::Optimism,
                settings.chains.optimism.url.clone()
            )),
            Chain::Aptos => Box::new(AptosClient::new(
                client,
                settings.chains.aptos.url.clone()
            )),
            Chain::Base => Box::new(EthereumClient::new(
                Chain::Base,
                settings.chains.base.url.clone()
            )),
            Chain::AvalancheC => Box::new(EthereumClient::new(
                Chain::AvalancheC,
                settings.chains.avalanchec.url.clone()
            )),
            Chain::Sui => Box::new(SuiClient::new(
                settings.chains.sui.url.clone()
            )),
            Chain::Ripple => Box::new(XRPClient::new(
                client,
                settings.chains.xrp.url.clone()
            )),
            Chain::OpBNB => Box::new(EthereumClient::new(
                Chain::OpBNB,
                settings.chains.opbnb.url.clone()
            )),
        }
    }
}

