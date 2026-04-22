use crate::{PriceProvider, SwapProvider};
use strum::AsRefStr;

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "camelCase")]
pub enum ConfigParamKey {
    SwapperVaultAddresses(SwapProvider),
    PriceProviderAssetsDuration(PriceProvider),
    PriceProviderAssetsNewDuration(PriceProvider),
    PriceProviderAssetsMetadataDuration(PriceProvider),
    PriceProviderPricesDuration(PriceProvider),
    PriceProviderChartsHourlyDuration(PriceProvider),
}

impl ConfigParamKey {
    pub fn all() -> Vec<Self> {
        let swapper = SwapProvider::cross_chain_providers().into_iter().map(Self::SwapperVaultAddresses);
        let assets = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsDuration);
        let assets_new = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsNewDuration);
        let assets_metadata = PriceProvider::all().into_iter().map(Self::PriceProviderAssetsMetadataDuration);
        let prices = PriceProvider::all().into_iter().map(Self::PriceProviderPricesDuration);
        let charts_hourly = PriceProvider::all().into_iter().map(Self::PriceProviderChartsHourlyDuration);
        swapper.chain(assets).chain(assets_new).chain(assets_metadata).chain(prices).chain(charts_hourly).collect()
    }

    pub fn key(&self) -> String {
        match self {
            Self::SwapperVaultAddresses(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsNewDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderAssetsMetadataDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderPricesDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
            Self::PriceProviderChartsHourlyDuration(provider) => format!("{}.{}", self.as_ref(), provider.as_ref()),
        }
    }

    pub fn default_value(&self) -> &str {
        match self {
            Self::SwapperVaultAddresses(_) => "5m",
            Self::PriceProviderAssetsDuration(_) => "1d",
            Self::PriceProviderAssetsNewDuration(_) => "15m",
            Self::PriceProviderAssetsMetadataDuration(_) => "30d",
            Self::PriceProviderPricesDuration(_) => "60s",
            Self::PriceProviderChartsHourlyDuration(_) => "7d",
        }
    }
}
