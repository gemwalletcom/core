use crate::{
    ESTIMATION_GAS_BUDGET, SUI_COIN_TYPE, SuiClient,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    models::{Coin, Gas, OwnedCoins, TokenTransferInput, TransferInput},
    tx_builder::{encode_token_transfer, encode_transfer},
};
use futures::try_join;
use num_traits::ToPrimitive;
use std::error::Error;

#[allow(clippy::too_many_arguments)]
pub async fn build_transfer_message_bytes(
    client: &SuiClient,
    sender: &str,
    recipient: &str,
    amount: u64,
    token_type: Option<&str>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (gas_price_bigint, sui_coins) = try_join!(client.get_gas_price(), client.get_coins(sender, SUI_COIN_TYPE))?;

    let gas_price = gas_price_bigint
        .to_u64()
        .ok_or_else(|| format!("Failed to convert Sui gas price to u64: {gas_price_bigint}"))?;

    if sui_coins.coins.is_empty() {
        return Err("No SUI coins available for gas budget".into());
    }

    let token_coins = match token_type {
        None => None,
        Some(token_type) => Some(get_token_coins(client, sender, token_type).await?),
    };

    let estimate_output = build_tx_output(sender, recipient, amount, &sui_coins, token_coins.as_ref(), ESTIMATION_GAS_BUDGET, gas_price)?;
    let dry_run_result = client.dry_run(estimate_output.base64_encoded()).await?;
    let fee = dry_run_result.effects.gas_used.calculate_gas_budget()?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;

    let tx_output = build_tx_output(sender, recipient, amount, &sui_coins, token_coins.as_ref(), gas_budget, gas_price)?;
    Ok(tx_output.base64_encoded())
}

async fn get_token_coins(client: &SuiClient, sender: &str, token_type: &str) -> Result<OwnedCoins<Coin>, Box<dyn Error + Send + Sync>> {
    let owned = client.get_coins(sender, token_type).await?;
    if owned.coins.is_empty() && owned.address_balance == 0 {
        return Err(format!("No coins or address balance found for token type {token_type}").into());
    }
    Ok(owned)
}

fn build_tx_output(
    sender: &str,
    recipient: &str,
    amount: u64,
    sui_coins: &OwnedCoins<Coin>,
    token_coins: Option<&OwnedCoins<Coin>>,
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
                tokens: tokens.clone(),
                gas,
                gas_coin: sui_coins.coins.first().unwrap().clone(),
            };
            encode_token_transfer(&token_transfer_input)
        }
        None => {
            let transfer_input = TransferInput {
                sender: sender.to_string(),
                recipient: recipient.to_string(),
                amount,
                coins: sui_coins.clone(),
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
        decode_base64(&message)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_build_transfer_message_bytes_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let message = build_transfer_message_bytes(&client, TEST_ADDRESS, TEST_ADDRESS, 1, Some(TEST_TOKEN_ADDRESS)).await?;
        decode_base64(&message)?;
        Ok(())
    }
}
