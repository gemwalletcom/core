use std::time::SystemTime;

use crate::model::{filter_token_id, FiatProviderAsset};
use hex;
use primitives::{Chain, FiatBuyRequest, FiatProviderName, FiatQuote};
use reqwest::Client;

use super::model::{
    Asset, Coins, Order, OrderData, OrderDetails, OrderRequest, Price, Prices, Response,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub struct BanxaClient {
    pub client: Client,
    pub url: String,
    pub merchant_key: String,
    pub secret_key: String,
}

impl BanxaClient {
    pub const NAME: FiatProviderName = FiatProviderName::Banxa;

    pub fn new(client: Client, url: String, merchant_key: String, secret_key: String) -> Self {
        BanxaClient {
            client,
            url,
            merchant_key,
            secret_key,
        }
    }

    pub fn get_authorization(&self, method: &str, query: &str, data: Option<&str>) -> String {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload = match data {
            Some(data) => format!("{}\n{}\n{}\n{}", method, query, nonce, data),
            None => format!("{}\n{}\n{}", method, query, nonce),
        };
        Self::generate_hmac(&self.merchant_key, &self.secret_key, &payload, nonce)
    }

    pub fn generate_hmac(
        merchant_key: &str,
        secret_key: &str,
        payload: &str,
        nonce: u64,
    ) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();
        format!("{}:{}:{}", merchant_key, hex::encode(signature), nonce)
    }

    pub async fn get_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let query = "/api/coins/buy";
        let authorization = self.get_authorization("GET", query, None);
        let url = format!("{}{}", self.url, query);
        let response = self
            .client
            .get(&url)
            .bearer_auth(authorization)
            .send()
            .await?
            .json::<Response<Coins>>()
            .await?;
        Ok(response.data.coins)
    }

    pub async fn get_quote_buy(
        &self,
        request: OrderRequest,
    ) -> Result<Order, Box<dyn std::error::Error + Send + Sync>> {
        let query = "/api/orders";
        let data = serde_json::to_string(&request)?;
        let authorization = self.get_authorization("POST", query, Some(&data));
        let url = format!("{}{}", self.url, query);
        let quote = self
            .client
            .post(&url)
            .bearer_auth(authorization)
            .json(&request)
            .send()
            .await?
            .json::<Response<OrderData<Order>>>()
            .await?;
        Ok(quote.data.order)
    }

    pub async fn get_order(
        &self,
        order_id: &str,
    ) -> Result<OrderDetails, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("/api/orders/{}", order_id);
        let authorization = self.get_authorization("GET", &query, None);
        let url = format!("{}{}", self.url, query);
        let response = self
            .client
            .get(&url)
            .bearer_auth(authorization)
            .send()
            .await?
            .json::<Response<OrderData<OrderDetails>>>()
            .await?;
        Ok(response.data.order)
    }

    pub async fn get_prices(
        &self,
        source: &str,
        target: &str,
    ) -> Result<Prices, Box<dyn std::error::Error + Send + Sync>> {
        let query = format!("/api/prices?source={}&target={}", source, target);
        let authorization = self.get_authorization("GET", &query, None);
        let url = format!("{}{}", self.url, query);
        let response = self
            .client
            .get(&url)
            .bearer_auth(authorization)
            .send()
            .await?
            .json::<Response<Prices>>()
            .await?;
        Ok(response.data)
    }

    pub fn map_asset(asset: Asset) -> Vec<FiatProviderAsset> {
        asset
            .clone()
            .blockchains
            .into_iter()
            .map(|blockchain| {
                let chain = Self::map_asset_chain(blockchain.clone().code.clone());
                let token_id = filter_token_id(blockchain.clone().contract_id);
                let id = asset.clone().coin_code + "-" + blockchain.clone().code.as_str();
                FiatProviderAsset {
                    id,
                    chain,
                    token_id,
                    symbol: asset.clone().coin_code.clone(),
                    network: Some(blockchain.code),
                    enabled: true,
                }
            })
            .collect()
    }

    pub fn map_asset_chain(chain: String) -> Option<Chain> {
        match chain.as_str() {
            "BTC" => Some(Chain::Bitcoin),
            "ETH" => Some(Chain::Ethereum),
            "TRON" => Some(Chain::Tron),
            "BSC" => Some(Chain::SmartChain),
            "SOL" => Some(Chain::Solana),
            "MATIC" => Some(Chain::Polygon),
            "ATOM " => Some(Chain::Cosmos),
            "AVAX-C" => Some(Chain::AvalancheC),
            "XRP" => Some(Chain::Xrp),
            "LTC" => Some(Chain::Litecoin),
            "FTM" => Some(Chain::Fantom),
            "DOGE" => Some(Chain::Doge),
            "APT" => Some(Chain::Aptos),
            "TON" => Some(Chain::Ton),
            "SUI" => Some(Chain::Sui),
            "NEAR" => Some(Chain::Near),
            "CELO" => Some(Chain::Celo),
            _ => None,
        }
    }

    pub fn get_fiat_quote(&self, request: FiatBuyRequest, price: Price, order: Order) -> FiatQuote {
        let price_fiat_amount = price.fiat_amount.parse::<f64>().unwrap_or_default();
        let fee_amount = price.fee_amount.parse::<f64>().unwrap_or_default();
        let network_fee = price.network_fee.parse::<f64>().unwrap_or_default();
        let crypto_amount = request.fiat_amount / (price_fiat_amount + fee_amount + network_fee);

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount,
            redirect_url: order.checkout_url,
        }
    }
}
