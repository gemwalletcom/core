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
    pub fn new_buy(
        partner_user_id: String,
        wallet_address: String,
        crypto_currency: String,
        fiat_currency: String,
        quote_id: String,
        user_ip: String,
        locale: String,
    ) -> Self {
        Self {
            partner_user_id,
            crypto_wallet_address: CryptoWalletAddress {
                address: wallet_address,
                currency_code: crypto_currency.clone(),
            },
            currency_code_from: fiat_currency,
            currency_code_to: crypto_currency,
            quote_id,
            user_ip,
            locale,
            flow: "buyCrypto".to_string(),
        }
    }

    pub fn new_sell(
        partner_user_id: String,
        wallet_address: String,
        crypto_currency: String,
        fiat_currency: String,
        quote_id: String,
        user_ip: String,
        locale: String,
    ) -> Self {
        Self {
            partner_user_id,
            crypto_wallet_address: CryptoWalletAddress {
                address: wallet_address,
                currency_code: crypto_currency.clone(),
            },
            currency_code_from: crypto_currency,
            currency_code_to: fiat_currency,
            quote_id,
            user_ip,
            locale,
            flow: "sellCrypto".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_buy_serialization() {
        let request = Request::new_buy(
            "test-user-id".to_string(),
            "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H".to_string(),
            "SOL".to_string(),
            "USD".to_string(),
            "test-quote-id".to_string(),
            "1.2.3.4".to_string(),
            "en".to_string(),
        );

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["partnerUserId"], "test-user-id");
        assert_eq!(parsed["cryptoWalletAddress"]["address"], "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H");
        assert_eq!(parsed["cryptoWalletAddress"]["currencyCode"], "SOL");
        assert_eq!(parsed["currencyCodeFrom"], "USD");
        assert_eq!(parsed["currencyCodeTo"], "SOL");
        assert_eq!(parsed["quoteId"], "test-quote-id");
        assert_eq!(parsed["userIp"], "1.2.3.4");
        assert_eq!(parsed["locale"], "en");
        assert_eq!(parsed["flow"], "buyCrypto");
    }

    #[test]
    fn test_request_sell_serialization() {
        let request = Request::new_sell(
            "test-user-id".to_string(),
            "0x1234567890abcdef".to_string(),
            "ETH".to_string(),
            "USD".to_string(),
            "test-quote-id".to_string(),
            "1.2.3.4".to_string(),
            "en".to_string(),
        );

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["partnerUserId"], "test-user-id");
        assert_eq!(parsed["cryptoWalletAddress"]["address"], "0x1234567890abcdef");
        assert_eq!(parsed["cryptoWalletAddress"]["currencyCode"], "ETH");
        assert_eq!(parsed["currencyCodeFrom"], "ETH");
        assert_eq!(parsed["currencyCodeTo"], "USD");
        assert_eq!(parsed["flow"], "sellCrypto");
    }
}
