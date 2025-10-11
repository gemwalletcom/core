use crate::{
    SUI_COIN_TYPE, SuiClient,
    models::{Coin, Gas, TokenTransferInput, TransferInput},
    operations::{encode_token_transfer, encode_transfer},
    provider::preload_mapper::GAS_BUDGET,
};
use futures::try_join;
use gem_client::Client;
use num_traits::ToPrimitive;
use std::error::Error;

/// Builds a base64 encoded programmable transaction payload for Sui transfers.
///
/// When `token_type` is `None`, a native SUI transfer is constructed. Otherwise the provided
/// token type is used to fetch token coins and construct a token transfer transaction.
#[allow(clippy::too_many_arguments)]
pub async fn build_transfer_message_bytes<C: Client + Clone>(
    client: &SuiClient<C>,
    sender: &str,
    recipient: &str,
    amount: u64,
    token_type: Option<&str>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (gas_price_bigint, sui_coin_objects) = try_join!(client.get_gas_price(), client.get_coins(sender, SUI_COIN_TYPE))?;

    let gas_price = gas_price_bigint
        .to_u64()
        .ok_or_else(|| format!("Failed to convert Sui gas price to u64: {gas_price_bigint}"))?;

    let sui_coins: Vec<Coin> = sui_coin_objects.into_iter().map(Into::into).collect();

    if sui_coins.is_empty() {
        return Err("No SUI coins available for gas budget".into());
    }

    let gas = Gas {
        budget: GAS_BUDGET,
        price: gas_price,
    };

    let tx_output = match token_type {
        None => {
            let transfer_input = TransferInput {
                sender: sender.to_string(),
                recipient: recipient.to_string(),
                amount,
                coins: sui_coins.clone(),
                send_max: false,
                gas,
            };

            encode_transfer(&transfer_input)?
        }
        Some(token_type) => {
            let token_coin_objects = client.get_coins(sender, token_type).await?;
            let token_coins: Vec<Coin> = token_coin_objects.into_iter().map(Into::into).collect();

            if token_coins.is_empty() {
                return Err(format!("No coins found for token type {token_type}").into());
            }

            let gas_coin = sui_coins
                .first()
                .cloned()
                .ok_or_else(|| "No SUI coins available to use as gas coin".to_string())
                .map_err(|err| -> Box<dyn Error + Send + Sync> { err.into() })?;

            let token_transfer_input = TokenTransferInput {
                sender: sender.to_string(),
                recipient: recipient.to_string(),
                amount,
                tokens: token_coins,
                gas,
                gas_coin,
            };

            encode_token_transfer(&token_transfer_input)?
        }
    };

    Ok(tx_output.base64_encoded())
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TOKEN_ADDRESS, create_sui_test_client};
    use base64::{Engine, engine::general_purpose};

    #[tokio::test]
    async fn test_build_transfer_message_bytes_native() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let message = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS, 1, None).await?;
        let (payload, digest) = message.split_once('_').ok_or("Missing digest separator")?;
        general_purpose::STANDARD.decode(payload)?;
        hex::decode(digest)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_build_transfer_message_bytes_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let message = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS, 1, Some(TEST_TOKEN_ADDRESS)).await?;
        let (payload, digest) = message.split_once('_').ok_or("Missing digest separator")?;
        general_purpose::STANDARD.decode(payload)?;
        hex::decode(digest)?;
        Ok(())
    }
}
