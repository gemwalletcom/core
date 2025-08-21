use async_trait::async_trait;
use chain_traits::ChainPreload;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::transaction_load::FeeOption;
use primitives::{
    AssetSubtype, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput,
};
use std::collections::HashMap;
use std::error::Error;

use crate::rpc::client::TonClient;
use crate::rpc::model::JettonWalletsResponse;

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
                options.insert(FeeOption::TokenAccountCreation, jetton_fee.to_string());
                &base_fee + jetton_fee
            }
        },
        TransactionInputType::Swap(_, _) => {
            let jetton_fee = BigInt::from(JETTON_ACCOUNT_CREATION);
            options.insert(FeeOption::TokenAccountCreation, jetton_fee.to_string());
            &base_fee + jetton_fee
        }
        _ => base_fee.clone(),
    };

    TransactionFee {
        fee,
        gas_price: base_fee,
        gas_limit: BigInt::from(1u64),
        options,
    }
}

fn check_jetton_account_exists(jetton_wallets: &JettonWalletsResponse, token_id: &str) -> bool {
    jetton_wallets.jetton_wallets.iter().any(|wallet| {
        crate::address::hex_to_base64_address(wallet.jetton.clone())
            .map(|address| address == *token_id)
            .unwrap_or(false)
    })
}

#[async_trait]
impl<C: Client> ChainPreload for TonClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let wallet_info = self.get_wallet_information(input.sender_address.clone()).await?;
        let sequence = wallet_info.seqno.unwrap_or(0) as u64;

        // For preload, we need to gather jetton wallet information if needed
        // Since we don't have transaction details in TransactionPreloadInput, we'll use empty string
        // The actual jetton wallet address will be determined based on transaction type during load
        let jetton_wallet_address = String::new();

        Ok(TransactionLoadMetadata::Ton {
            jetton_wallet_address,
            sequence,
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        // Extract metadata and determine if we need to fetch additional jetton data
        let (jetton_wallet_address, account_exists) = self.get_jetton_info_for_transaction(&input).await?;
        let fee = calculate_transaction_fee(&input, account_exists);

        // Use the sequence from the metadata passed in
        let sequence = input.metadata.get_sequence()?;
        let metadata = TransactionLoadMetadata::Ton {
            jetton_wallet_address,
            sequence,
        };

        Ok(TransactionLoadData { fee, metadata })
    }
}

impl<C: Client> TonClient<C> {
    async fn get_jetton_info_for_transaction(&self, input: &TransactionLoadInput) -> Result<(String, bool), Box<dyn Error + Sync + Send>> {
        match &input.input_type {
            TransactionInputType::Transfer(asset) => match asset.id.token_subtype() {
                AssetSubtype::TOKEN => {
                    let token_id = asset.id.token_id.as_ref().ok_or("Missing token ID")?;
                    let jetton_token_id = crate::address::base64_to_hex_address(token_id.clone())?.to_uppercase();

                    let jetton_wallets = self.get_jetton_wallets(input.sender_address.clone()).await?;

                    let jetton_wallet_address = jetton_wallets
                        .jetton_wallets
                        .iter()
                        .find(|wallet| wallet.jetton == jetton_token_id)
                        .map(|wallet| wallet.address.clone())
                        .ok_or_else(|| format!("Jetton wallet not found for token {}", jetton_token_id))?;

                    let account_exists = check_jetton_account_exists(&jetton_wallets, token_id);
                    Ok((jetton_wallet_address, account_exists))
                }
                AssetSubtype::NATIVE => Ok(("".to_string(), true)),
            },
            TransactionInputType::Swap(_, to_asset) => {
                if let Some(token_id) = &to_asset.id.token_id {
                    let jetton_wallets = self.get_jetton_wallets(input.sender_address.clone()).await?;
                    let jetton_token_id = crate::address::base64_to_hex_address(token_id.clone())?.to_uppercase();

                    let jetton_wallet_address = jetton_wallets
                        .jetton_wallets
                        .iter()
                        .find(|wallet| wallet.jetton == jetton_token_id)
                        .map(|wallet| wallet.address.clone())
                        .ok_or_else(|| format!("Jetton wallet not found for token {}", token_id))?;

                    let account_exists = check_jetton_account_exists(&jetton_wallets, token_id);
                    Ok((jetton_wallet_address, account_exists))
                } else {
                    Ok(("".to_string(), true))
                }
            }
            TransactionInputType::Stake(_, _) => Ok(("".to_string(), true)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain, GasPrice};

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
            gas_price: GasPrice {
                gas_price: BigInt::from(10_000_000u64),
            },
            memo,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                jetton_wallet_address: "".to_string(),
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
            Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING).to_string())
        );
    }

    #[test]
    fn test_jetton_existing_account_with_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), true);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO).to_string())
        );
    }

    #[test]
    fn test_jetton_new_account() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, None), false);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_CREATION).to_string())
        );
    }

    #[test]
    fn test_jetton_new_account_ignores_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), false);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_CREATION).to_string())
        );
    }
}
