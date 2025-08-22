use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::transaction_load::FeeOption;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};
use std::collections::HashMap;
use std::error::Error;

use crate::rpc::client::TonClient;

const TON_BASE_FEE: u64 = 10_000_000;
const JETTON_ACCOUNT_FEE_EXISTING: u64 = 100_000_000;
const JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO: u64 = 60_000_000;
const JETTON_ACCOUNT_CREATION: u64 = 200_000_000;

pub fn calculate_transaction_fee(input: &TransactionLoadInput, account_exists: bool) -> TransactionFee {
    let base_fee = BigInt::from(TON_BASE_FEE);
    let mut options = HashMap::new();

    let fee = match &input.input_type {
        TransactionInputType::Transfer(asset) => match asset.id.token_subtype() {
            AssetSubtype::NATIVE => base_fee.clone(),
            AssetSubtype::TOKEN => {
                let jetton_fee = if account_exists {
                    if input.memo.is_some() {
                        BigInt::from(JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO)
                    } else {
                        BigInt::from(JETTON_ACCOUNT_FEE_EXISTING)
                    }
                } else {
                    BigInt::from(JETTON_ACCOUNT_CREATION)
                };
                options.insert(FeeOption::TokenAccountCreation, jetton_fee.clone());
                &base_fee + jetton_fee
            }
        },
        TransactionInputType::Swap(_, _) => {
            options.insert(FeeOption::TokenAccountCreation, BigInt::from(JETTON_ACCOUNT_CREATION));
            &base_fee + BigInt::from(JETTON_ACCOUNT_CREATION)
        }
        _ => base_fee.clone(),
    };

    TransactionFee::new_from_fee(fee)
}

#[async_trait]
impl<C: Client> ChainTransactionLoad for TonClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let wallet_info = self.get_wallet_information(input.sender_address.clone()).await?;
        let sequence = wallet_info.seqno.unwrap_or(0) as u64;

        return match &input.asset.id.token_subtype() {
            AssetSubtype::TOKEN => {
                let token_id = input.asset.id.token_id.as_ref().ok_or("Missing token ID for jetton transaction")?;
                let jetton_token_id = crate::address::base64_to_hex_address(token_id.clone())?.to_uppercase();

                let jetton_wallets = self.get_jetton_wallets(input.sender_address.clone()).await?;

                let jetton_wallet_address = jetton_wallets
                    .jetton_wallets
                    .iter()
                    .find(|wallet| wallet.jetton == jetton_token_id)
                    .map(|wallet| wallet.address.clone())
                    .ok_or_else(|| format!("Jetton wallet not found for token {}", jetton_token_id))?;

                Ok(TransactionLoadMetadata::Ton {
                    jetton_wallet_address: Some(jetton_wallet_address),
                    sequence,
                })
            }
            AssetSubtype::NATIVE => Ok(TransactionLoadMetadata::Ton {
                jetton_wallet_address: None,
                sequence,
            }),
        };
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = calculate_transaction_fee(&input, input.metadata.get_is_destination_address_exist()?);

        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(10000000))), // 0.01 TON
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, AssetType, Chain, GasPriceType};

    fn create_input(asset_type: AssetType, memo: Option<String>) -> TransactionLoadInput {
        let (token_id, name, symbol, decimals) = match asset_type {
            AssetType::NATIVE => (None, "TON".to_string(), "TON".to_string(), 9),
            AssetType::JETTON => (Some("test_token".to_string()), "Test Token".to_string(), "TEST".to_string(), 6),
            _ => panic!("Unsupported asset type"),
        };

        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset {
                id: AssetId {
                    chain: Chain::Ton,
                    token_id: token_id.clone(),
                },
                chain: Chain::Ton,
                token_id,
                name,
                symbol,
                decimals,
                asset_type,
            }),
            sender_address: "test".to_string(),
            destination_address: "test".to_string(),
            value: "1000".to_string(),
            gas_price: GasPriceType::regular(BigInt::from(10_000_000u64)),
            memo,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                jetton_wallet_address: None,
                sequence: 0,
            },
        }
    }

    #[test]
    fn test_native_ton() {
        let fee = calculate_transaction_fee(&create_input(AssetType::NATIVE, None), true);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE));
        assert_eq!(fee.options.len(), 0);
    }

    #[test]
    fn test_native_ton_with_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::NATIVE, Some("memo".to_string())), true);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE));
        assert_eq!(fee.options.len(), 0);
    }

    #[test]
    fn test_jetton_existing_account() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, None), true);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_FEE_EXISTING));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING))
        );
    }

    #[test]
    fn test_jetton_existing_account_with_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), true);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO))
        );
    }

    #[test]
    fn test_jetton_new_account() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, None), false);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_CREATION)));
    }

    #[test]
    fn test_jetton_new_account_ignores_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), false);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_CREATION)));
    }
}
