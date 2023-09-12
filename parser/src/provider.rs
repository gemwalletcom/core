use blockchain::{ChainProvider, BNBChainClient, SolanaClient};
use primitives::Chain;
use settings::Settings;
trait ProviderFactory {
    fn new(&self, chain: Chain, settings: &Settings) -> Box<dyn ChainProvider>;
}

pub fn new(chain: Chain, settings: &Settings) -> Box<dyn ChainProvider> {
    match chain {
        Chain::Bitcoin => todo!(),
        Chain::Ethereum => todo!(),
        Chain::Binance => {
            Box::new(BNBChainClient::new(
                settings.chains.binance.url.clone(),
                settings.chains.binance.api.clone(),
            ))
        },
        Chain::SmartChain => todo!(),
        Chain::Solana => {
            Box::new(SolanaClient::new(settings.chains.solana.url.clone()))
        },
        Chain::Polygon => todo!(),
        Chain::Thorchain => todo!(),
        Chain::Cosmos => todo!(),
        Chain::Osmosis => todo!(),
        Chain::Arbitrum => todo!(),
        Chain::Ton => todo!(),
        Chain::Tron => todo!(),
        Chain::Doge => todo!(),
        Chain::Optimism => todo!(),
        Chain::Aptos => todo!(),
        Chain::Base => todo!(),
        Chain::AvalancheC => todo!(),
        Chain::Sui => todo!(),
        Chain::Ripple => todo!(),
        Chain::OpBNB => todo!(),
    }
}