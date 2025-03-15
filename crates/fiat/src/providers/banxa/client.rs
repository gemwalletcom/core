use std::time::SystemTime;

use crate::model::{filter_token_id, FiatMapping, FiatProviderAsset};
use hex;
use primitives::{FiatProviderName, FiatQuote, FiatQuoteRequest};
use reqwest::Client;
use url::Url;

use super::model::{Asset, Coins, OrderData, OrderDetails, Price, Prices, Response};
use hmac::{Hmac, Mac};
use primitives::FiatTransactionType;
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
        let nonce = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let payload = match data {
            Some(data) => format!("{}\n{}\n{}\n{}", method, query, nonce, data),
            None => format!("{}\n{}\n{}", method, query, nonce),
        };
        Self::generate_hmac(&self.merchant_key, &self.secret_key, &payload, nonce)
    }

    pub fn generate_hmac(merchant_key: &str, secret_key: &str, payload: &str, nonce: u64) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();
        format!("{}:{}:{}", merchant_key, hex::encode(signature), nonce)
    }

    pub async fn get_buy_assets(&self) -> Result<Vec<Asset>, Box<dyn std::error::Error + Send + Sync>> {
        let query = "/api/coins/buy";
        let authorization = self.get_authorization("GET", query, None);
        let url = format!("{}{}", self.url, query);
        Ok(self
            .client
            .get(&url)
            .bearer_auth(authorization)
            .send()
            .await?
            .json::<Response<Coins>>()
            .await?
            .data
            .coins)
    }

    pub async fn get_order(&self, order_id: &str) -> Result<OrderDetails, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn get_prices(&self, source: &str, target: &str) -> Result<Prices, Box<dyn std::error::Error + Send + Sync>> {
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
                let chain = super::mapper::map_asset_chain(blockchain.clone().code.clone());
                let token_id = filter_token_id(chain, blockchain.clone().contract_id);
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

    pub fn get_fiat_buy_quote(&self, request: FiatQuoteRequest, fiat_mapping: FiatMapping, price: Price) -> FiatQuote {
        let crypto_amount = request.fiat_amount / (price.fiat_amount + price.fee_amount + price.network_fee);
        let redirect_url = self.get_redirect_buy_url(request.clone(), fiat_mapping);

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatTransactionType::Buy,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount,
            redirect_url,
        }
    }

    pub fn get_fiat_sell_quote(&self, request: FiatQuoteRequest, fiat_mapping: FiatMapping, _price: Price) -> FiatQuote {
        let crypto_amount = request.crypto_amount.unwrap_or_default();
        //request.fiat_amount / (price.fiat_amount + price.fee_amount + price.network_fee);
        let redirect_url = self.get_redirect_sell_url(request.clone(), fiat_mapping);

        FiatQuote {
            provider: Self::NAME.as_fiat_provider(),
            quote_type: FiatTransactionType::Sell,
            fiat_amount: request.fiat_amount,
            fiat_currency: request.fiat_currency,
            crypto_amount,
            redirect_url,
        }
    }

    // URL Parametization https://docs.banxa.com/docs/referral-link

    pub fn get_redirect_buy_url(&self, request: FiatQuoteRequest, fiat_mapping: FiatMapping) -> String {
        let mut components = Url::parse(&self.url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("orderType", "buy")
            .append_pair("coinType", &fiat_mapping.symbol)
            .append_pair("blockchain", &fiat_mapping.network.unwrap_or_default())
            .append_pair("fiatType", request.fiat_currency.as_str())
            .append_pair("fiatAmount", &request.fiat_amount.to_string())
            .append_pair("walletAddress", &request.wallet_address);
        components.as_str().to_string()
    }

    pub fn get_redirect_sell_url(&self, request: FiatQuoteRequest, fiat_mapping: FiatMapping) -> String {
        let mut components = Url::parse(&self.url).unwrap();
        components
            .query_pairs_mut()
            .append_pair("orderType", "sell")
            .append_pair("coinType", &fiat_mapping.symbol)
            .append_pair("blockchain", &fiat_mapping.network.unwrap_or_default())
            .append_pair("fiatType", request.fiat_currency.as_str())
            .append_pair("coinAmount", &request.crypto_amount.unwrap_or_default().to_string())
            .append_pair("walletAddress", &request.wallet_address);
        components.as_str().to_string()
    }
}
