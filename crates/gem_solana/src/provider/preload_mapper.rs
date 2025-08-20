use num_bigint::BigInt;
use primitives::{AssetSubtype, FeePriority, FeeRate, GasPrice, SignerInputToken, SolanaTokenProgramId, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput};
use primitives::transaction_load::TransactionLoadMetadata;
use std::collections::HashMap;

use crate::{models::prioritization_fee::SolanaPrioritizationFee, model::ValueResult, model::TokenAccountInfo, get_token_program_id_by_address};

const STATIC_BASE_FEE: u64 = 5000;

pub fn calculate_transaction_fee(
    input_type: &TransactionInputType,
    gas_price: &GasPrice,
    prioritization_fees: &[SolanaPrioritizationFee],
) -> TransactionFee {
    let gas_limit = get_gas_limit(input_type);
    let priority_fee = calculate_priority_fee(input_type, prioritization_fees);
    
    let total_fee = BigInt::from(STATIC_BASE_FEE) + &gas_price.gas_price + priority_fee;

    TransactionFee {
        fee: total_fee,
        gas_price: gas_price.gas_price.clone(),
        gas_limit: BigInt::from(gas_limit),
        options: HashMap::new(),
    }
}

pub fn calculate_priority_fee(
    input_type: &TransactionInputType,
    prioritization_fees: &[SolanaPrioritizationFee],
) -> BigInt {
    // Filter out large fees and get top 5
    let mut fees: Vec<i64> = prioritization_fees.iter().map(|f| f.prioritization_fee).collect();
    fees.sort_by(|a, b| b.cmp(a)); // Sort descending
    fees.truncate(5);

    let multiple_of = get_multiple_of(input_type);
    
    let priority_fee_base = if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    };

    // For normal priority (could be extended for slow/fast)
    priority_fee_base
}

fn get_gas_limit(input_type: &TransactionInputType) -> u64 {
    match input_type {
        TransactionInputType::Transfer(_) => 100_000,
        TransactionInputType::Swap(_, _) => 420_000,
        TransactionInputType::Stake(_) => 100_000,
    }
}

fn get_multiple_of(input_type: &TransactionInputType) -> i64 {
    match input_type {
        TransactionInputType::Transfer(asset) => {
            match &asset.id.token_subtype() {
                AssetSubtype::NATIVE => 25_000,
                AssetSubtype::TOKEN => 50_000, 
            }
        },
        TransactionInputType::Stake(_) => 25_000,
        TransactionInputType::Swap(_, _) => 100_000,
    }
}

fn round_to_nearest(value: i64, multiple: i64, round_up: bool) -> i64 {
    if round_up {
        ((value + multiple - 1) / multiple) * multiple
    } else {
        (value / multiple) * multiple
    }
}

pub fn calculate_fee_rates(
    input_type: &TransactionInputType,
    prioritization_fees: &[SolanaPrioritizationFee],
) -> Vec<FeeRate> {
    let mut fees: Vec<i64> = prioritization_fees.iter().map(|f| f.prioritization_fee).collect();
    fees.sort_by(|a, b| b.cmp(a));
    fees.truncate(5);

    let multiple_of = get_multiple_of(input_type);
    let gas_limit = get_gas_limit(input_type);
    let static_base_fee = BigInt::from(STATIC_BASE_FEE);
    
    let priority_fee_base = if fees.is_empty() {
        BigInt::from(multiple_of)
    } else {
        let average = fees.iter().sum::<i64>() / fees.len() as i64;
        let rounded = round_to_nearest(average, multiple_of, true);
        BigInt::from(std::cmp::max(rounded, multiple_of))
    };

    vec![
        FeeRate::eip1559(
            FeePriority::Slow, 
            static_base_fee.clone(),
            (&priority_fee_base / 2_i64 * BigInt::from(gas_limit)) / BigInt::from(1_000_000u64)
        ),
        FeeRate::eip1559(
            FeePriority::Normal, 
            static_base_fee.clone(),
            (&priority_fee_base * BigInt::from(gas_limit)) / BigInt::from(1_000_000u64)
        ),
        FeeRate::eip1559(
            FeePriority::Fast, 
            static_base_fee,
            (&priority_fee_base * 3_i64 * BigInt::from(gas_limit)) / BigInt::from(1_000_000u64)
        ),
    ]
}

pub fn map_transaction_load(
    input: TransactionLoadInput,
    prioritization_fees: Vec<SolanaPrioritizationFee>,
    token_accounts: Option<(ValueResult<Vec<TokenAccountInfo>>, ValueResult<Vec<TokenAccountInfo>>)>,
) -> TransactionLoadData {
    let fee = calculate_transaction_fee(&input.input_type, &input.gas_price, &prioritization_fees);
    
    let metadata = match &input.input_type {
        TransactionInputType::Transfer(asset) => {
            match &asset.id.token_id {
                Some(_) => {
                    if let Some((sender_accounts, recipient_accounts)) = token_accounts {
                        let token_info = map_token_transfer_info(sender_accounts, recipient_accounts);
                        Some(TransactionLoadMetadata::Solana {
                            sender_token_address: token_info.sender_token_address,
                            recipient_token_address: token_info.recipient_token_address,
                            token_program: token_info.token_program,
                            sequence: input.sequence,
                        })
                    } else {
                        Some(TransactionLoadMetadata::Solana {
                            sender_token_address: String::new(),
                            recipient_token_address: None,
                            token_program: SolanaTokenProgramId::Token,
                            sequence: input.sequence,
                        })
                    }
                },
                None => Some(TransactionLoadMetadata::Solana {
                    sender_token_address: String::new(),
                    recipient_token_address: None,
                    token_program: SolanaTokenProgramId::Token,
                    sequence: input.sequence,
                }),
            }
        },
        _ => Some(TransactionLoadMetadata::Solana {
            sender_token_address: String::new(),
            recipient_token_address: None,
            token_program: SolanaTokenProgramId::Token,
            sequence: input.sequence,
        }),
    };

    TransactionLoadData {
        fee,
        metadata: metadata.unwrap_or(TransactionLoadMetadata::Solana {
            sender_token_address: String::new(),
            recipient_token_address: None,
            token_program: SolanaTokenProgramId::Token,
            sequence: input.sequence,
        }),
    }
}

fn map_token_transfer_info(
    sender_accounts: ValueResult<Vec<TokenAccountInfo>>,
    recipient_accounts: ValueResult<Vec<TokenAccountInfo>>,
) -> SignerInputToken {
    let sender_token_address = sender_accounts.value.first()
        .map(|account| account.pubkey.clone())
        .unwrap_or_default();
    
    let token_program = sender_accounts.value.first()
        .and_then(|account| get_token_program_id(&account.account.owner))
        .unwrap_or(SolanaTokenProgramId::Token);
    
    let recipient_token_address = recipient_accounts.value.first()
        .map(|account| account.pubkey.clone());

    SignerInputToken {
        sender_token_address,
        recipient_token_address,
        token_program,
    }
}

fn get_token_program_id(owner: &str) -> Option<SolanaTokenProgramId> {
    get_token_program_id_by_address(owner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, Asset, AssetType};

    #[test]
    fn test_calculate_transaction_fee() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 100_000 }];
        let gas_price = GasPrice { gas_price: BigInt::from(1000u64) };
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let fee = calculate_transaction_fee(&input_type, &gas_price, &fees);
        assert!(fee.fee > BigInt::from(STATIC_BASE_FEE));
    }

    #[test] 
    fn test_calculate_priority_fee() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 150_000 }];
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let priority_fee = calculate_priority_fee(&input_type, &fees);
        assert_eq!(priority_fee, BigInt::from(150_000));
    }

    #[test]
    fn test_calculate_fee_rates() {
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 100_000 }];
        let input_type = TransactionInputType::Transfer(Asset {
            id: AssetId::from_chain(Chain::Solana),
            chain: Chain::Solana,
            token_id: None,
            name: "SOL".to_string(),
            symbol: "SOL".to_string(),
            decimals: 9,
            asset_type: AssetType::NATIVE,
        });

        let rates = calculate_fee_rates(&input_type, &fees);
        assert_eq!(rates.len(), 3);
    }

    #[test]
    fn test_map_transaction_load() {
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset {
                id: AssetId::from_chain(Chain::Solana),
                chain: Chain::Solana,
                token_id: None,
                name: "SOL".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                asset_type: AssetType::NATIVE,
            }),
            gas_price: GasPrice { gas_price: BigInt::from(1000u64) },
            sequence: 123,
            sender_address: "sender".to_string(),
            destination_address: "dest".to_string(),
            value: "1000000".to_string(),
            block_hash: "blockhash".to_string(),
            block_number: 12345,
            chain_id: "solana".to_string(),
            utxos: vec![],
            memo: None,
        };
        let fees = vec![SolanaPrioritizationFee { prioritization_fee: 50_000 }];

        let result = map_transaction_load(input, fees, None);

        if let TransactionLoadMetadata::Solana { sequence, .. } = result.metadata {
            assert_eq!(sequence, 123);
        } else {
            panic!("Expected Solana metadata");
        }
        assert!(result.fee.fee > BigInt::from(0));
    }
}