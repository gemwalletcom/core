use crate::client::ThorchainSwapClient;
use async_trait::async_trait;
use primitives::{Chain, SwapQuote, SwapQuoteProtocolRequest};
use swap_provider::{SwapError, SwapProvider};

pub const PROVIDER_NAME: &str = "1inch";

pub struct ThorchainProvider {
    pub client: ThorchainSwapClient,
}

impl ThorchainProvider {
    pub fn new(api_url: String, fee: f64, fee_referral_address: String) -> Self {
        Self {
            client: ThorchainSwapClient::new(api_url, fee, fee_referral_address),
        }
    }

    pub fn new_box(api_url: String, fee: f64, fee_referral_address: String) -> Box<Self> {
        Box::new(Self::new(api_url, fee, fee_referral_address))
    }
}

#[async_trait]
impl SwapProvider for ThorchainProvider {
    fn provider(&self) -> primitives::SwapProvider {
        PROVIDER_NAME.into()
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![
            Chain::Bitcoin,
            Chain::Litecoin,
            Chain::Cosmos,
            // Chain::SmartChain, disable for now
            Chain::Doge,
            Chain::Thorchain,
        ]
    }

    async fn get_quote(&self, request: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
        let quote = self.client.get_quote(request).await?;
        Ok(quote)
    }
}
