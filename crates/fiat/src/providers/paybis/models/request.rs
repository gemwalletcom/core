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
    pub requested_amount: f64,
    pub requested_amount_type: String,
    pub user_ip: String,
    pub locale: String,
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
        fiat_amount: f64,
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
            requested_amount: fiat_amount,
            requested_amount_type: "from".to_string(),
            user_ip,
            locale,
        }
    }

    pub fn new_sell(
        partner_user_id: String,
        wallet_address: String,
        crypto_currency: String,
        fiat_currency: String,
        crypto_amount: f64,
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
            requested_amount: crypto_amount,
            requested_amount_type: "from".to_string(),
            user_ip,
            locale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request {
            partner_user_id: "test-user-id".to_string(),
            crypto_wallet_address: CryptoWalletAddress {
                address: "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H".to_string(),
                currency_code: "SOL".to_string(),
            },
            currency_code_from: "USD".to_string(),
            currency_code_to: "SOL".to_string(),
            requested_amount: 50.0,
            requested_amount_type: "from".to_string(),
            user_ip: "1.2.3.4".to_string(),
            locale: "en".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["partnerUserId"], "test-user-id");
        assert_eq!(parsed["cryptoWalletAddress"]["address"], "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H");
        assert_eq!(parsed["cryptoWalletAddress"]["currencyCode"], "SOL");
        assert_eq!(parsed["currencyCodeFrom"], "USD");
        assert_eq!(parsed["currencyCodeTo"], "SOL");
        assert_eq!(parsed["requestedAmount"], 50.0);
        assert_eq!(parsed["requestedAmountType"], "from");
        assert_eq!(parsed["userIp"], "1.2.3.4");
        assert_eq!(parsed["locale"], "en");
    }
}
