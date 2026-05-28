use std::error::Error;

use gem_encoding::encode_base64;
use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType, StakeType, TransactionInputType, TransactionLoadInput};

use crate::{encode_split_and_stake, encode_token_transfer, encode_transfer, encode_unstake, models::*, validate_and_hash};

pub fn map_transaction_rate_rates(base_gas_price: BigInt) -> Vec<FeeRate> {
    vec![
        FeeRate::new(FeePriority::Slow, GasPriceType::regular(base_gas_price.clone())),
        FeeRate::new(FeePriority::Normal, GasPriceType::regular(&base_gas_price * BigInt::from(110) / BigInt::from(100))),
        FeeRate::new(FeePriority::Fast, GasPriceType::regular(&base_gas_price * BigInt::from(2))),
    ]
}

pub fn map_transaction_data(
    input: TransactionLoadInput,
    sui_coins: OwnedCoins<Coin>,
    token_coins: Option<OwnedCoins<Coin>>,
    objects: Vec<SuiObject>,
    gas_budget: u64,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let gas_price = input.gas_price.gas_price().to_string().parse().unwrap_or(0);

    match input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => match asset.id.token_id.as_ref() {
            None => {
                let transfer_input = TransferInput {
                    sender: input.sender_address,
                    recipient: input.destination_address,
                    amount: input.value.parse().unwrap_or(0),
                    coins: sui_coins,
                    send_max: input.is_max_value,
                    gas: Gas {
                        budget: gas_budget,
                        price: gas_price,
                    },
                };
                let tx_output = encode_transfer(&transfer_input)?;
                let data = encode_base64(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            Some(_token_id) => {
                let gas_coin = sui_coins.coins.first().ok_or("No gas coins available for token transfer")?.clone();
                let tokens = token_coins.ok_or("Missing token coins set for Sui token transfer")?;
                let token_transfer_input = TokenTransferInput {
                    sender: input.sender_address,
                    recipient: input.destination_address,
                    amount: input.value.parse().unwrap_or(0),
                    tokens,
                    gas: Gas {
                        budget: gas_budget,
                        price: gas_price,
                    },
                    gas_coin,
                };
                let tx_output = encode_token_transfer(&token_transfer_input)?;
                let data = encode_base64(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
        },
        TransactionInputType::Stake(_, stake_type) => match stake_type {
            StakeType::Stake(validator) => {
                let stake_input = StakeInput {
                    sender: input.sender_address,
                    validator: validator.id.clone(),
                    stake_amount: input.value.parse().unwrap_or(0),
                    gas: Gas {
                        budget: gas_budget,
                        price: gas_price,
                    },
                    coins: sui_coins,
                };
                let tx_output = encode_split_and_stake(&stake_input)?;
                let data = encode_base64(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            StakeType::Unstake(delegation) => {
                let gas_coin = sui_coins.coins.first().ok_or("No gas coins available for unstake")?.clone();
                let staked_object = objects
                    .iter()
                    .find(|obj| obj.object_id == delegation.base.delegation_id)
                    .ok_or("Staked SUI object not found in provided objects")?;

                let staked_sui = crate::models::Object {
                    object_id: staked_object.object_id.parse().map_err(|err| format!("invalid staked Sui object id: {err}"))?,
                    version: staked_object.version.parse().unwrap_or(0),
                    digest: staked_object.digest.parse().map_err(|err| format!("invalid staked Sui object digest: {err}"))?,
                };

                let unstake_input = UnstakeInput {
                    sender: input.sender_address,
                    staked_sui,
                    gas: Gas {
                        budget: gas_budget,
                        price: gas_price,
                    },
                    gas_coin,
                };
                let tx_output = encode_unstake(&unstake_input)?;
                let data = encode_base64(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => {
                Err("Unsupported stake type for Sui".into())
            }
        },
        TransactionInputType::Swap(_, _, data) => {
            let tx_output = validate_and_hash(&data.data.data)?;
            let data = encode_base64(&tx_output.tx_data);
            let digest = hex::encode(&tx_output.hash);
            Ok(format!("{}_{}", data, digest))
        }
        TransactionInputType::Generic(_, _, extra) => {
            let raw_data = extra.data.ok_or("Missing transaction data for Sui generic input")?;
            let encoded = String::from_utf8(raw_data).map_err(|_| "Invalid UTF-8 data for Sui generic input")?;
            let tx_output = validate_and_hash(&encoded)?;
            let data = encode_base64(&tx_output.tx_data);
            let digest = hex::encode(&tx_output.hash);
            Ok(format!("{}_{}", data, digest))
        }
        _ => Err("Unsupported transaction type for Sui".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SuiObject;
    use primitives::{Asset, Chain, Delegation, StakeType, TransactionInputType, TransactionLoadInput};

    #[test]
    fn test_unstake_uses_object_reference() {
        let delegation_id = "0x1234567890abcdef1234567890abcdef12345678";
        let delegation = Delegation::mock_with_id(delegation_id.to_string());
        let input_type = TransactionInputType::Stake(Asset::from_chain(Chain::Sui), StakeType::Unstake(delegation));
        let input = TransactionLoadInput::mock_with_input_type(input_type);

        let gas_coins = OwnedCoins::<Coin>::mock_sui();

        let objects = vec![SuiObject {
            object_id: delegation_id.to_string(),
            version: "12345".to_string(),
            digest: "CU86BjXRF1XHFRjKBasCYEuaQxhHuyGBpuoJyqsrYoX5".to_string(),
        }];

        let result = map_transaction_data(input, gas_coins, None, objects, 25_000_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_map_transaction_rate_rates() {
        let rates = map_transaction_rate_rates(BigInt::from(497));

        assert_eq!(rates.len(), 3);
        assert_eq!(rates[0].priority, FeePriority::Slow);
        assert_eq!(rates[0].gas_price_type.gas_price(), BigInt::from(497));
        assert_eq!(rates[1].priority, FeePriority::Normal);
        assert_eq!(rates[1].gas_price_type.gas_price(), BigInt::from(546));
        assert_eq!(rates[2].priority, FeePriority::Fast);
        assert_eq!(rates[2].gas_price_type.gas_price(), BigInt::from(994));
    }
}
