pub mod client;
pub use self::client::SwapperClient;
use swap_aftermath::provider::AftermathProvider;
use swap_oneinch::OneInchClient;
use swap_provider::ProviderList;
pub mod jupiter;
pub use self::jupiter::JupiterClient;
pub mod thorswap;
pub use self::thorswap::ThorchainSwapClient;
pub use swap_skip_client::provider::SkipProvider;

pub struct SwapperConfiguration {
    pub oneinch: SwapperClientConfiguration,
    pub jupiter: SwapperClientConfiguration,
    pub thorchain: SwapperClientConfiguration,
    pub aftermath: SwapperClientConfiguration,
    pub skip: SwapperClientConfiguration,
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
        let thorchain_swap_client = ThorchainSwapClient::new(
            configuration.thorchain.url,
            configuration.thorchain.fee_percent,
            configuration.thorchain.fee_address,
        );

        let providers: ProviderList = vec![
            Box::new(AftermathProvider::new(
                configuration.aftermath.fee_address,
                configuration.aftermath.fee_percent as f32,
            )),
            Box::new(SkipProvider::new(
                configuration.skip.key, // client_id
                (configuration.skip.fee_percent * 100.0) as u32,
                configuration.skip.fee_address,
            )),
        ];

        SwapperClient::new(
            oneinch_client,
            jupiter_client,
            thorchain_swap_client,
            providers,
        )
    }
}
