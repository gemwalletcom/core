use crate::{encode_split_and_stake, encode_token_transfer, encode_transfer, encode_unstake, models::*, validate_and_hash};
use base64::{engine::general_purpose, Engine};
use num_bigint::BigInt;
use primitives::{
    transaction_load_metadata::SuiCoin, FeePriority, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadInput,
    TransactionLoadMetadata,
};
use std::{collections::HashMap, error::Error};

pub const GAS_BUDGET: u64 = 25_000_000;

pub fn calculate_transaction_fee(input_type: &TransactionInputType, gas_price_type: &GasPriceType) -> TransactionFee {
    let gas_limit = get_gas_limit(input_type);
    let fee = get_fee(input_type);

    TransactionFee {
        fee,
        gas_price_type: gas_price_type.clone(),
        gas_limit: BigInt::from(gas_limit),
        options: HashMap::new(),
    }
}

pub fn calculate_fee_rates(base_gas_price: BigInt) -> Vec<FeeRate> {
    vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(base_gas_price))]
}

fn get_gas_limit(input_type: &TransactionInputType) -> u64 {
    match input_type {
        TransactionInputType::Transfer(_)
        | TransactionInputType::Deposit(_)
        | TransactionInputType::TokenApprove(_, _)
        | TransactionInputType::Generic(_, _, _)
        | TransactionInputType::Perpetual(_, _) => GAS_BUDGET,
        TransactionInputType::Swap(_, _, _) => 50_000_000,
        TransactionInputType::Stake(_, _) => GAS_BUDGET,
    }
}

fn get_fee(_input_type: &TransactionInputType) -> BigInt {
    BigInt::from(GAS_BUDGET)
}

impl From<SuiCoin> for crate::models::Coin {
    fn from(coin: SuiCoin) -> Self {
        crate::models::Coin {
            coin_type: coin.coin_type,
            balance: coin.balance.to_string().parse().unwrap_or(0),
            object: crate::models::Object {
                object_id: coin.coin_object_id,
                digest: coin.digest,
                version: coin.version.parse().unwrap_or(0),
            },
        }
    }
}

pub fn map_preload_metadata(_coins: Vec<SuiCoin>) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
    Ok(TransactionLoadMetadata::Sui { message_bytes: "".to_string() })
}

pub fn map_transaction_data(input: TransactionLoadInput, gas_coins: Vec<SuiCoin>, coins: Vec<SuiCoin>) -> Result<String, Box<dyn Error + Send + Sync>> {
    let gas_coins: Vec<Coin> = gas_coins.into_iter().map(Into::into).collect();
    let coins: Vec<Coin> = coins.into_iter().map(Into::into).collect();

    let gas_price = input.gas_price.gas_price().to_string().parse().unwrap_or(0);

    match input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => match asset.id.token_id.as_ref() {
            None => {
                let transfer_input = TransferInput {
                    sender: input.sender_address,
                    recipient: input.destination_address,
                    amount: input.value.parse().unwrap_or(0),
                    coins: gas_coins,
                    send_max: false,
                    gas: Gas {
                        budget: GAS_BUDGET,
                        price: gas_price,
                    },
                };
                let tx_output = encode_transfer(&transfer_input)?;
                let data = general_purpose::STANDARD.encode(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            Some(_token_id) => {
                if gas_coins.is_empty() {
                    return Err("No gas coins available for token transfer".into());
                }
                let gas_coin = gas_coins.first().unwrap().clone();
                let token_transfer_input = TokenTransferInput {
                    sender: input.sender_address,
                    recipient: input.destination_address,
                    amount: input.value.parse().unwrap_or(0),
                    tokens: coins,
                    gas: Gas {
                        budget: GAS_BUDGET,
                        price: gas_price,
                    },
                    gas_coin,
                };
                let tx_output = encode_token_transfer(&token_transfer_input)?;
                let data = general_purpose::STANDARD.encode(&tx_output.tx_data);
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
                        budget: GAS_BUDGET,
                        price: gas_price,
                    },
                    coins: gas_coins,
                };
                let tx_output = encode_split_and_stake(&stake_input)?;
                let data = general_purpose::STANDARD.encode(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            StakeType::Unstake(delegation) => {
                if gas_coins.is_empty() {
                    return Err("No gas coins available for unstake".into());
                }
                let gas_coin = gas_coins.first().unwrap().clone();

                let staked_sui = crate::models::Object {
                    object_id: delegation.base.delegation_id.clone(),
                    version: 1,
                    digest: "".to_string(),
                };

                let unstake_input = UnstakeInput {
                    sender: input.sender_address,
                    staked_sui,
                    gas: Gas {
                        budget: GAS_BUDGET,
                        price: gas_price,
                    },
                    gas_coin,
                };
                let tx_output = encode_unstake(&unstake_input)?;
                let data = general_purpose::STANDARD.encode(&tx_output.tx_data);
                let digest = hex::encode(&tx_output.hash);
                Ok(format!("{}_{}", data, digest))
            }
            StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => Err("Unsupported stake type for Sui".into()),
        },
        TransactionInputType::Swap(_, _, data) => {
            let tx_output = validate_and_hash(&data.data.data)?;
            let data = general_purpose::STANDARD.encode(&tx_output.tx_data);
            let digest = hex::encode(&tx_output.hash);
            Ok(format!("{}_{}", data, digest))
        }
        _ => Err("Unsupported transaction type for Sui".into()),
    }
}
