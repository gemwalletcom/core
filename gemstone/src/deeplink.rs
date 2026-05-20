use primitives::{AssetId, Deeplink};

#[uniffi::remote(Enum)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Swap { from_asset_id: AssetId, to_asset_id: Option<AssetId> },
    Perpetuals,
    Rewards { code: Option<String> },
    Gift { code: Option<String> },
    Buy { asset_id: AssetId, amount: Option<i32> },
    Sell { asset_id: AssetId, amount: Option<i32> },
    SetPriceAlert { asset_id: AssetId, price: Option<f64> },
}

#[uniffi::export]
pub fn deeplink_build_url(deeplink: Deeplink) -> String {
    deeplink.to_url()
}

#[uniffi::export]
pub fn deeplink_build_gem_url(deeplink: Deeplink) -> String {
    deeplink.to_gem_url()
}

#[uniffi::export]
pub fn deeplink_decode_url(url: &str) -> Option<Deeplink> {
    Deeplink::from_url(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deeplink() {
        let rewards = Deeplink::Rewards {
            code: Some("gemcoder".to_string()),
        };
        assert_eq!(deeplink_build_url(rewards.clone()), "https://gemwallet.com/rewards?code=gemcoder");
        assert_eq!(deeplink_build_gem_url(Deeplink::Perpetuals), "gem://perpetuals");
        assert_eq!(deeplink_decode_url("https://gemwallet.com/rewards?code=gemcoder"), Some(rewards));
        assert_eq!(deeplink_decode_url("https://gemwallet.com/unknown"), None);
    }
}
