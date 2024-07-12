pub mod client;
pub use crate::client::SwapperClient;
use swap_aftermath::provider::AftermathProvider;
use swap_jupiter::client::JupiterClient;
use swap_oneinch::OneInchClient;
use swap_provider::ProviderList;
use swap_thorchain::provider::ThorchainProvider;

pub struct SwapperConfiguration {
    pub oneinch: SwapperClientConfiguration,
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

pub struct SwapperOneinchConfiguration {}

pub struct Swapper {}

impl Swapper {
    pub fn build(configuration: SwapperConfiguration) -> SwapperClient {
        let oneinch_client = OneInchClient::new(
            configuration.oneinch.url,
            configuration.oneinch.key,
            configuration.oneinch.fee_percent,
            configuration.oneinch.fee_address,
        );
        let jupiter_client = JupiterClient::new(
            configuration.jupiter.url,
            configuration.jupiter.fee_percent,
            configuration.jupiter.fee_address,
        );

        let providers: ProviderList = vec![
            AftermathProvider::new_box(
                configuration.aftermath.fee_address,
                configuration.aftermath.fee_percent as f32,
            ),
            ThorchainProvider::new_box(
                configuration.thorchain.url,
                configuration.thorchain.fee_percent,
                configuration.thorchain.fee_address,
            ),
        ];

        SwapperClient::new(oneinch_client, jupiter_client, providers)
    }
}
