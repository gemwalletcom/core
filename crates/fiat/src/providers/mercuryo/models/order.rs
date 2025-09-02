use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Deserialize, Clone)]
pub struct Order {
    pub merchant_transaction_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MercuryoTransactionResponse {
    pub buy: Option<BuyTransaction>,
    pub mobile_pay: Option<MobilePayTransaction>,
    pub sell: Option<SellTransaction>,
    pub withdraw: Option<WithdrawTransaction>,
    pub deposit: Option<DepositTransaction>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BuyTransaction {
    pub merchant_transaction_id: String,
    pub fiat_currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    pub currency: String,
    pub status: String,
    pub card_country: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WithdrawTransaction {
    pub merchant_transaction_id: String,
    pub hash: Option<String>,
    pub address: Option<String>,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MobilePayTransaction {
    pub merchant_transaction_id: String,
    pub fiat_currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    pub currency: String,
    pub status: String,
    pub payment_method: Option<String>,
    pub card_country: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SellTransaction {
    pub merchant_transaction_id: String,
    pub fiat_currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub fiat_amount: f64,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DepositTransaction {
    pub merchant_transaction_id: String,
    pub hash: Option<String>,
    pub address: Option<String>,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub amount: f64,
    pub status: String,
}
