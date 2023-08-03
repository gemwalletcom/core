#[typeshare(swift = "Equatable, Codable, Hashable")]
struct Price {
    price: f64,
    #[serde(rename = "priceChangePercentage24h")]
    price_change_percentage_24h: f64,
}