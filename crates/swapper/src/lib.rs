pub mod client;
pub use crate::client::SwapperClient;
use swap_aftermath::provider::AftermathProvider;
use swap_jupiter::client::JupiterClient;
use swap_provider::ProviderList;
use swap_thorchain::provider::ThorchainProvider;

pub struct SwapperConfiguration {
    pub jupiter: SwapperClientConfiguration,
    pub thorchain: SwapperClientConfiguration,
    pub aftermath: SwapperClientConfiguration,
}

pub struct SwapperClientConfiguration {
    pub url: String,
    pub key: String,
    pub fee_percent: f64,
    pub fee_address: String,
}

pub struct Swapper {}

impl Swapper {
    pub fn build(configuration: SwapperConfiguration) -> SwapperClient {
        let jupiter_client = JupiterClient::new(configuration.jupiter.url, configuration.jupiter.fee_percent, configuration.jupiter.fee_address);

        let providers: ProviderList = vec![
            Box::new(AftermathProvider::new(
                configuration.aftermath.fee_address,
                configuration.aftermath.fee_percent as f32,
            )),
            Box::new(ThorchainProvider::new(
                configuration.thorchain.url,
                configuration.thorchain.fee_percent,
                configuration.thorchain.fee_address,
            )),
        ];

        SwapperClient::new(jupiter_client, providers)
    }
}
