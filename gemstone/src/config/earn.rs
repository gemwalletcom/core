use std::collections::HashMap;

pub fn get_underlying_assets_by_provider(provider_id: &str) -> HashMap<String, Vec<String>> {
    yielder::get_underlying_assets_by_provider(provider_id)
        .into_iter()
        .map(|(backed_asset, underlying_assets)| (backed_asset.to_string(), underlying_assets.into_iter().map(|asset| asset.to_string()).collect()))
        .collect()
}
