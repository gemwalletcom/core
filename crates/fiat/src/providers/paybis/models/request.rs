use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CryptoWalletAddress {
    pub address: String,
    pub currency_code: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub partner_user_id: String,
    pub partner_transaction_id: Option<String>,
    pub crypto_wallet_address: CryptoWalletAddress,
    pub currency_code_from: String,
    pub currency_code_to: String,
    pub quote_id: String,
    pub user_ip: String,
    pub locale: String,
    pub flow: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestResponse {
    pub request_id: String,
}

impl Request {
    fn new(
        partner_user_id: String,
        wallet_address: String,
        wallet_currency_code: String,
        currency_code_from: String,
        currency_code_to: String,
        quote_id: String,
        user_ip: String,
        locale: String,
        flow: String,
    ) -> Self {
        Self {
            partner_user_id,
            partner_transaction_id: Some(quote_id.clone()),
            crypto_wallet_address: CryptoWalletAddress {
                address: wallet_address,
                currency_code: wallet_currency_code,
            },
            currency_code_from,
            currency_code_to,
            quote_id,
            user_ip,
            locale,
            flow,
        }
    }

    pub fn new_sell(partner_user_id: String, wallet_address: String, crypto_currency: String, fiat_currency: String, quote_id: String, user_ip: String, locale: String) -> Self {
        Self::new(
            partner_user_id,
            wallet_address,
            crypto_currency.clone(),
            crypto_currency,
            fiat_currency,
            quote_id,
            user_ip,
            locale,
            "sellCrypto".to_string(),
        )
    }

    pub fn new_buy(partner_user_id: String, wallet_address: String, crypto_currency: String, fiat_currency: String, quote_id: String, user_ip: String, locale: String) -> Self {
        Self::new(
            partner_user_id,
            wallet_address,
            crypto_currency.clone(),
            fiat_currency,
            crypto_currency,
            quote_id,
            user_ip,
            locale,
            "buyCrypto".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_buy_serializes_flow_and_currency_direction() {
        let request = Request::new_buy(
            "test-user-id".to_string(),
            "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H".to_string(),
            "SOL".to_string(),
            "USD".to_string(),
            "test-quote-id".to_string(),
            "1.2.3.4".to_string(),
            "en".to_string(),
        );

        let parsed = serde_json::to_value(&request).unwrap();

        assert_eq!(parsed["cryptoWalletAddress"]["currencyCode"], "SOL");
        assert_eq!(parsed["currencyCodeFrom"], "USD");
        assert_eq!(parsed["currencyCodeTo"], "SOL");
        assert_eq!(parsed["partnerTransactionId"], "test-quote-id");
        assert_eq!(parsed["quoteId"], "test-quote-id");
        assert_eq!(parsed["flow"], "buyCrypto");
    }

    #[test]
    fn request_sell_serializes_flow_and_currency_direction() {
        let request = Request::new_sell(
            "test-user-id".to_string(),
            "0x1234567890abcdef".to_string(),
            "ETH".to_string(),
            "USD".to_string(),
            "test-quote-id".to_string(),
            "1.2.3.4".to_string(),
            "en".to_string(),
        );

        let parsed = serde_json::to_value(&request).unwrap();

        assert_eq!(parsed["cryptoWalletAddress"]["currencyCode"], "ETH");
        assert_eq!(parsed["currencyCodeFrom"], "ETH");
        assert_eq!(parsed["currencyCodeTo"], "USD");
        assert_eq!(parsed["partnerTransactionId"], "test-quote-id");
        assert_eq!(parsed["quoteId"], "test-quote-id");
        assert_eq!(parsed["flow"], "sellCrypto");
    }
}
