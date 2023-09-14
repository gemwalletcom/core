use blockchain::{ChainProvider, BNBChainClient, SolanaClient, EthereumClient};
use primitives::Chain;
use settings::Settings;
trait ProviderFactory {
    fn new(&self, chain: Chain, settings: &Settings) -> Box<dyn ChainProvider>;
}

pub fn new(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
    match chain {
        Chain::Bitcoin => todo!(),
        Chain::Ethereum => Box::new(EthereumClient::new(
            Chain::Ethereum,
            settings.chains.ethereum.url.clone()
        )),
        Chain::Binance => Box::new(BNBChainClient::new(
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
        Chain::Thorchain => todo!(),
        Chain::Cosmos => todo!(),
        Chain::Osmosis => todo!(),
        Chain::Arbitrum => Box::new(EthereumClient::new(
            Chain::Arbitrum,
            settings.chains.arbitrum.url.clone()
        )),
        Chain::Ton => todo!(),
        Chain::Tron => todo!(),
        Chain::Doge => todo!(),
        Chain::Optimism => Box::new(EthereumClient::new(
            Chain::Optimism,
            settings.chains.optimism.url.clone()
        )),
        Chain::Aptos => todo!(),
        Chain::Base => Box::new(EthereumClient::new(
            Chain::Base,
            settings.chains.base.url.clone()
        )),
        Chain::AvalancheC => Box::new(EthereumClient::new(
            Chain::AvalancheC,
            settings.chains.avalanchec.url.clone()
        )),
        Chain::Sui => todo!(),
        Chain::Ripple => todo!(),
        Chain::OpBNB => Box::new(EthereumClient::new(
            Chain::OpBNB,
            settings.chains.opbnb.url.clone()
        )),
    }
}