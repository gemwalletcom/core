#[typeshare]
#[serde(rename_all = "camelCase")]
struct AssetPrice {
    asset_id: String,
    price: f64,
    price_change_percentage_24h: f64,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct AssetPrices {
    currency: String,
    prices: Vec<AssetPrice>,
}

#[typeshare(swift = "Equatable")]
#[serde(rename_all = "camelCase")]
struct AssetPricesRequest {
    currency: String,
    asset_ids: Vec<String>,
}

