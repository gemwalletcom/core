#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
struct FiatQuote {
    provider: FiatProvider,
    fiat_amount: f64,
    fiat_currency: String,
    crypto_amount: f64,
    redirect_url: String,
}

#[typeshare()]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>
}