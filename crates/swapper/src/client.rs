use primitives::{Chain, SwapQuote, SwapQuoteProtocolRequest};
use swap_oneinch::OneInchClient;
use swap_provider::ProviderList;

use crate::{JupiterClient, ThorchainSwapClient};

pub struct SwapperClient {
    oneinch: OneInchClient,
    jupiter: JupiterClient,
    thorchain: ThorchainSwapClient,
    providers: ProviderList,
}

impl SwapperClient {
    pub fn new(
        oneinch: OneInchClient,
        jupiter: JupiterClient,
        thorchain: ThorchainSwapClient,
        providers: ProviderList,
    ) -> Self {
        Self {
            oneinch,
            jupiter,
            thorchain,
            providers,
        }
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let source_chain = quote.from_asset.chain;
        let result = self
            .providers
            .iter()
            .find(|x| x.supported_chains().contains(&source_chain));
        if let Some(provider) = result {
            return provider.get_quote(quote).await;
        }

        match source_chain {
            Chain::Ethereum
            | Chain::SmartChain
            | Chain::Optimism
            | Chain::Arbitrum
            | Chain::Polygon
            | Chain::Base
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::AvalancheC
            | Chain::Manta
            | Chain::Blast
            | Chain::ZkSync
            | Chain::Linea
            | Chain::Mantle
            | Chain::Celo => self.oneinch.get_quote(quote).await,
            Chain::Solana => self.jupiter.get_quote(quote).await,
            Chain::Thorchain | Chain::Doge | Chain::Cosmos | Chain::Bitcoin | Chain::Litecoin => {
                self.thorchain.get_quote(quote).await
            }
            Chain::Osmosis
            | Chain::Celestia
            | Chain::Injective
            | Chain::Ton
            | Chain::Tron
            | Chain::Aptos
            | Chain::Sui
            | Chain::Xrp
            | Chain::OpBNB
            | Chain::Noble
            | Chain::Sei
            | Chain::Near
            | Chain::Dymension
            | Chain::Saga => todo!(),
        }
    }
}
