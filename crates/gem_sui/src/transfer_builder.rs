use crate::{
    ESTIMATION_GAS_BUDGET, SUI_COIN_TYPE, SuiClient,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    models::{Coin, Gas, TokenTransferInput, TransferInput},
    operations::{encode_token_transfer, encode_transfer},
};
use futures::try_join;
use gem_client::Client;
use num_traits::ToPrimitive;
use std::error::Error;

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

    let token_coins = match token_type {
        None => None,
        Some(token_type) => Some(get_token_coins(client, sender, token_type).await?),
    };

    let estimate_output = build_tx_output(sender, recipient, amount, &sui_coins, token_coins.as_deref(), ESTIMATION_GAS_BUDGET, gas_price)?;
    let dry_run_result = client.dry_run(estimate_output.base64_encoded()).await?;
    let fee = dry_run_result.effects.gas_used.calculate_gas_budget()?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;

    let tx_output = build_tx_output(sender, recipient, amount, &sui_coins, token_coins.as_deref(), gas_budget, gas_price)?;
    Ok(tx_output.base64_encoded())
}

async fn get_token_coins<C: Client + Clone>(client: &SuiClient<C>, sender: &str, token_type: &str) -> Result<Vec<Coin>, Box<dyn Error + Send + Sync>> {
    let objs = client.get_coins(sender, token_type).await?;
    let coins: Vec<Coin> = objs.into_iter().map(Into::into).collect();
    if coins.is_empty() {
        return Err(format!("No coins found for token type {token_type}").into());
    }
    Ok(coins)
}

fn build_tx_output(
    sender: &str,
    recipient: &str,
    amount: u64,
    sui_coins: &[Coin],
    token_coins: Option<&[Coin]>,
    gas_budget: u64,
    gas_price: u64,
) -> Result<crate::models::TxOutput, Box<dyn Error + Send + Sync>> {
    let gas = Gas {
        budget: gas_budget,
        price: gas_price,
    };

    match token_coins {
        Some(tokens) => {
            let token_transfer_input = TokenTransferInput {
                sender: sender.to_string(),
                recipient: recipient.to_string(),
                amount,
                tokens: tokens.to_vec(),
                gas,
                gas_coin: sui_coins.first().unwrap().clone(),
            };
            encode_token_transfer(&token_transfer_input)
        }
        None => {
            let transfer_input = TransferInput {
                sender: sender.to_string(),
                recipient: recipient.to_string(),
                amount,
                coins: sui_coins.to_vec(),
                send_max: false,
                gas,
            };
            encode_transfer(&transfer_input)
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TOKEN_ADDRESS, create_sui_test_client};
    use gem_encoding::decode_base64;

    #[tokio::test]
    async fn test_build_transfer_message_bytes_native() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let message = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS, 1, None).await?;
        let (payload, digest) = message.split_once('_').ok_or("Missing digest separator")?;
        decode_base64(payload)?;
        hex::decode(digest)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_build_transfer_message_bytes_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let message = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS, 1, Some(TEST_TOKEN_ADDRESS)).await?;
        let (payload, digest) = message.split_once('_').ok_or("Missing digest separator")?;
        decode_base64(payload)?;
        hex::decode(digest)?;
        Ok(())
    }
}
