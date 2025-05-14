use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;

use super::client::PaybisClient; // Correctly path to PaybisClient in client.rs
use super::model::{PaybisQuoteRequest, PaybisWebhookPayload}; // Correctly path to PaybisQuoteRequest and PaybisWebhookPayload in model.rs
use crate::model::{FiatMapping, FiatProviderAsset}; // FiatMapping might be used for asset_id
use crate::provider::FiatProvider;
use primitives::{FiatBuyQuote, FiatProviderCountry, FiatProviderName, FiatQuote, FiatQuoteType, FiatSellQuote, FiatTransaction, FiatTransactionStatus};

#[async_trait]
impl FiatProvider for PaybisClient {
    fn name(&self) -> FiatProviderName {
        FiatProviderName::Paybis
    }

    async fn get_buy_quote(
        &self,
        request: FiatBuyQuote,
        request_map: FiatMapping, // Use for provider-specific asset ID
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let crypto_currency_to_buy = request_map.symbol.clone();

        let paybis_request = PaybisQuoteRequest {
            currency_code_from: request.fiat_currency.clone(),
            currency_code_to: crypto_currency_to_buy,
            amount_from: request.fiat_amount.to_string(),
        };

        let quote_response = self
            .get_quote(paybis_request)
            .await
            .map_err(|e| format!("Paybis API error: {}", e).to_string())?;

        let fiat_amount = quote_response
            .amount_from
            .parse::<f64>()
            .map_err(|e| format!("Paybis API error: failed to parse fiat_amount '{}': {}", quote_response.amount_from, e).to_string())?;
        let crypto_amount = quote_response
            .amount_to
            .parse::<f64>()
            .map_err(|e| format!("Paybis API error: failed to parse crypto_amount '{}': {}", quote_response.amount_to, e).to_string())?;

        Ok(FiatQuote {
            provider: self.name().as_fiat_provider(),
            quote_type: FiatQuoteType::Buy,
            fiat_amount,
            fiat_currency: quote_response.currency_code_from, // Use response's currency code
            crypto_amount,
            crypto_value: quote_response.currency_code_to, // This is the crypto currency symbol
            redirect_url: quote_response.redirect_url.unwrap_or_default(),
        })
    }

    async fn get_sell_quote(
        &self,
        request: FiatSellQuote,
        request_map: FiatMapping, // Use for provider-specific asset ID
    ) -> Result<FiatQuote, Box<dyn std::error::Error + Send + Sync>> {
        let crypto_currency_from_sell = request_map.symbol.clone();

        let paybis_request = PaybisQuoteRequest {
            currency_code_from: crypto_currency_from_sell,   // Crypto being sold
            currency_code_to: request.fiat_currency.clone(), // Fiat to receive
            amount_from: request.crypto_amount.to_string(),  // Amount of crypto to sell
        };

        let quote_response = self
            .get_quote(paybis_request)
            .await
            .map_err(|e| format!("Paybis API error: {}", e).to_string())?;

        // For a sell quote:
        // amount_from is the crypto amount confirmed by Paybis
        // amount_to is the fiat amount the user will receive
        let crypto_amount_sold = quote_response
            .amount_from
            .parse::<f64>()
            .map_err(|e| format!("Paybis API error: failed to parse crypto_amount_sold '{}': {}", quote_response.amount_from, e).to_string())?;
        let fiat_amount_received = quote_response
            .amount_to
            .parse::<f64>()
            .map_err(|e| format!("Paybis API error: failed to parse fiat_amount_received '{}': {}", quote_response.amount_to, e).to_string())?;

        Ok(FiatQuote {
            provider: self.name().as_fiat_provider(),
            quote_type: FiatQuoteType::Sell,
            fiat_amount: fiat_amount_received,
            fiat_currency: quote_response.currency_code_to, // Fiat currency user receives
            crypto_amount: crypto_amount_sold,
            crypto_value: quote_response.currency_code_from, // Crypto currency user sold
            redirect_url: quote_response.redirect_url.unwrap_or_default(),
        })
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let buy_pairs = self.get_buy_currency_pairs().await?;
        let sell_pairs = self.get_sell_currency_pairs().await?;

        let mut unique_crypto_asset_codes = HashSet::new();

        for pair in buy_pairs {
            // For buy-crypto pairs, currency_code_to is the crypto asset the user receives
            unique_crypto_asset_codes.insert(pair.currency_code_to);
        }

        for pair in sell_pairs {
            // For sell-crypto pairs, currency_code_from is the crypto asset the user sells
            unique_crypto_asset_codes.insert(pair.currency_code_from);
        }

        let assets = unique_crypto_asset_codes
            .into_iter()
            .map(|asset_code| FiatProviderAsset {
                id: asset_code.clone(), // Use the currency code as the ID
                symbol: asset_code,     // Use the currency code as the symbol
                chain: None,            // Paybis currency pair API does not provide chain info here
                token_id: None,         // No token_id from this Paybis endpoint
                network: None,          // No specific network string from this Paybis endpoint
                enabled: true,
                unsupported_countries: None, // This endpoint doesn't provide per-asset country restrictions
            })
            .collect();

        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        // Based on https://docs.payb.is/docs/supported-countries (accessed 2025-05-14)
        // This list represents countries/territories explicitly mentioned as prohibited.
        // US states and other nuanced restrictions (e.g. stablecoins in TX/CA) are not fully captured here
        // as FiatProviderCountry expects a simple alpha2 country code and is_allowed status.
        // "Geographies with unrecognized or disputed status" is also not codified here.
        let restricted_country_codes = vec![
            "AF", // Afghanistan
            "BY", // Belarus
            "CU", // Cuba
            "IR", // Iran
            "IQ", // Iraq
            "MM", // Myanmar
            "KP", // North Korea
            "RU", // Russia
            "SO", // Somalia
            "SS", // South Sudan
            "SD", // Sudan
            "SY", // Syria
                  // US States listed as generally restricted (not just for specific assets):
                  // Hawaii, Louisiana, New York. Representing these as 'US' with a note or handling
                  // them separately if our model supported states would be more accurate.
                  // For now, if a US state implies broader US restriction for Paybis, one might add "US".
                  // However, Paybis differentiates states, so simply listing "US" as restricted is inaccurate.
                  // The current model of FiatProviderCountry doesn't handle sub-regions like states well.
                  // We will return country-level restrictions here.
        ];

        let paybis_provider_id = self.name().id();

        let countries = restricted_country_codes
            .into_iter()
            .map(|code| FiatProviderCountry {
                provider: paybis_provider_id.clone(),
                alpha2: code.to_string(),
                is_allowed: false, // These are the *restricted* countries
            })
            .collect();

        Ok(countries)
    }

    async fn webhook(&self, data: Value) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let payload: PaybisWebhookPayload = serde_json::from_value(data)?;

        let (status, provider_transaction_id, symbol, transaction_hash, fiat_amount_str, fiat_currency_str) = match payload {
            PaybisWebhookPayload::Success(event) => {
                (
                    FiatTransactionStatus::Complete,
                    event.transaction_id,
                    event.digital_amount_sent.currency,
                    Some(event.blockchain_txn_hash),
                    event.digital_amount_sent.amount, // This is crypto amount, fiat amount not directly available
                    String::new(),                    // Fiat currency not directly available
                )
            }
            PaybisWebhookPayload::Error(event) => {
                // Determine status based on Paybis's status field, if possible, or default to Failed
                let tx_status = match event.status.to_lowercase().as_str() {
                    "rejected" => FiatTransactionStatus::Failed,
                    "error" => FiatTransactionStatus::Failed,
                    _ => FiatTransactionStatus::Failed, // Default for other error statuses
                };
                (
                    tx_status,
                    event.transaction_id,
                    event.amount_sent.currency,
                    None,
                    event.amount_sent.amount, // This is crypto amount, fiat amount not directly available
                    String::new(),            // Fiat currency not directly available
                )
            }
        };

        // Attempt to parse fiat_amount from string, default to 0.0 if parse fails or not applicable
        let fiat_amount = fiat_amount_str.parse::<f64>().unwrap_or(0.0);
        // Fiat currency might not be available from webhook, use default or what makes sense.
        // Paybis webhook examples focus on crypto amounts.

        let transaction = FiatTransaction {
            asset_id: None,                       // AssetId (Chain + TokenId) is not directly available from webhook
            transaction_type: FiatQuoteType::Buy, // Assume Buy for now, as webhooks are for payouts
            symbol,                               // Crypto symbol
            provider_id: self.name().id(),
            provider_transaction_id,
            status,
            country: None,                    // Country info not in webhook payload
            fiat_amount,                      // Fiat amount is not directly provided in crypto payout webhooks
            fiat_currency: fiat_currency_str, // Fiat currency is not directly provided
            transaction_hash,                 // Blockchain transaction hash if successful
            address: None,                    // Wallet address not in webhook payload
            fee_provider: None,               // Fee info not in webhook payload
            fee_network: None,                // Fee info not in webhook payload
            fee_partner: None,                // Fee info not in webhook payload
        };

        Ok(transaction)
    }
}
