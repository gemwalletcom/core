use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use gem_client::Client;
use num_bigint::BigInt;
use primitives::FeeOption;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, swap::SwapQuoteDataType,
};
use std::collections::HashMap;
use std::error::Error;

use crate::address::base64_to_hex_address;
use crate::rpc::client::TonClient;

const TON_BASE_FEE: u64 = 10_000_000;
const JETTON_ACCOUNT_FEE_EXISTING: u64 = 100_000_000;
const JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO: u64 = 60_000_000;
const JETTON_ACCOUNT_CREATION: u64 = 200_000_000;
const SWAP_NATIVE_RESERVE: u64 = 310_000_000;

pub fn calculate_transaction_fee(input: &TransactionLoadInput, recipient_token_address: Option<String>) -> TransactionFee {
    let base_fee = BigInt::from(TON_BASE_FEE);
    let mut options = HashMap::new();

    let fee = match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::TransferNft(asset, _) | TransactionInputType::Account(asset, _) => {
            transfer_fee(asset.id.token_subtype(), input.memo.as_deref(), recipient_token_address.as_deref(), &base_fee, &mut options)
        }
        TransactionInputType::Swap(from_asset, _, swap_data) => match &swap_data.data.data_type {
            SwapQuoteDataType::Contract => {
                options.insert(FeeOption::TokenAccountCreation, BigInt::from(SWAP_NATIVE_RESERVE));
                base_fee
            }
            SwapQuoteDataType::Transfer => transfer_fee(
                from_asset.id.token_subtype(),
                input.memo.as_deref(),
                recipient_token_address.as_deref(),
                &base_fee,
                &mut options,
            ),
        },
        TransactionInputType::TokenApprove(_, _) => base_fee.clone(),
        TransactionInputType::Generic(_, _, _) => base_fee.clone(),
        TransactionInputType::Perpetual(_, _) => base_fee.clone(),
        _ => base_fee.clone(),
    };

    TransactionFee::new_gas_price_type(GasPriceType::regular(fee.clone()), fee.clone(), BigInt::from(1), options)
}

fn transfer_fee(asset_subtype: AssetSubtype, memo: Option<&str>, recipient_token_address: Option<&str>, base_fee: &BigInt, options: &mut HashMap<FeeOption, BigInt>) -> BigInt {
    match asset_subtype {
        AssetSubtype::NATIVE => base_fee.clone(),
        AssetSubtype::TOKEN => {
            let jetton_fee = if recipient_token_address.is_some() {
                if memo.is_some() {
                    BigInt::from(JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO)
                } else {
                    BigInt::from(JETTON_ACCOUNT_FEE_EXISTING)
                }
            } else {
                BigInt::from(JETTON_ACCOUNT_CREATION)
            };
            options.insert(FeeOption::TokenAccountCreation, jetton_fee);
            base_fee.clone()
        }
    }
}

#[async_trait]
impl<C: Client> ChainTransactionLoad for TonClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let wallet_info = self.get_wallet_information(input.sender_address.clone()).await?;
        let sequence = wallet_info.seqno.unwrap_or(0) as u64;

        let asset = input.input_type.get_asset();
        return match &asset.id.token_subtype() {
            AssetSubtype::TOKEN => {
                let token_id = asset.id.token_id.as_ref().ok_or("Missing token ID for jetton transaction")?;
                let jetton_token_id = base64_to_hex_address(token_id).ok_or("Invalid jetton token ID")?.to_uppercase();

                let sender_wallets = self.get_jetton_wallets(input.sender_address.clone());
                let recipient_wallets = async {
                    match get_recipient_jetton_wallet(&input) {
                        Some(address) => self.get_jetton_wallets(address.to_string()).await.map(Some),
                        None => Ok(None),
                    }
                };
                let (sender_jetton_wallets, recipient_jetton_wallets) = futures::future::try_join(sender_wallets, recipient_wallets).await?;

                let sender_jetton_wallet_address = sender_jetton_wallets.jetton_wallets.iter().find(|wallet| wallet.jetton == jetton_token_id);
                let recipient_jetton_wallet_address = recipient_jetton_wallets
                    .as_ref()
                    .and_then(|wallets| wallets.jetton_wallets.iter().find(|wallet| wallet.jetton == jetton_token_id));

                Ok(TransactionLoadMetadata::Ton {
                    sender_token_address: sender_jetton_wallet_address.map(|x| x.address.clone()),
                    recipient_token_address: recipient_jetton_wallet_address.map(|x| x.address.clone()),
                    sequence,
                })
            }
            AssetSubtype::NATIVE => Ok(TransactionLoadMetadata::Ton {
                sender_token_address: None,
                recipient_token_address: None,
                sequence,
            }),
        };
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = calculate_transaction_fee(&input, input.metadata.get_recipient_token_address()?);

        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(10000000))), // 0.01 TON
        ])
    }
}

fn get_recipient_jetton_wallet(input: &TransactionPreloadInput) -> Option<&str> {
    match &input.input_type {
        TransactionInputType::Swap(_, _, swap_data) => match &swap_data.data.data_type {
            SwapQuoteDataType::Transfer => Some(&swap_data.data.to),
            SwapQuoteDataType::Contract => None,
        },
        _ => Some(&input.destination_address),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, AssetType, Chain, GasPriceType, SwapProvider, TransactionPreloadInput, swap::SwapData};

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
                sender_token_address: None,
                recipient_token_address: None,
                sequence: 0,
            },
        }
    }

    #[test]
    fn test_native_ton() {
        let fee = calculate_transaction_fee(&create_input(AssetType::NATIVE, None), None);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE));
        assert_eq!(fee.options.len(), 0);
    }

    #[test]
    fn test_native_ton_with_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::NATIVE, Some("memo".to_string())), None);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE));
        assert_eq!(fee.options.len(), 0);
    }

    #[test]
    fn test_jetton_existing_account() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, None), Some("existing_account".to_string()));
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_FEE_EXISTING));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING)));
    }

    #[test]
    fn test_jetton_existing_account_with_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), Some("existing_account".to_string()));
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO));
        assert_eq!(
            fee.options.get(&FeeOption::TokenAccountCreation),
            Some(&BigInt::from(JETTON_ACCOUNT_FEE_EXISTING_WITH_MEMO))
        );
    }

    #[test]
    fn test_jetton_new_account() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, None), None);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_CREATION)));
    }

    #[test]
    fn test_jetton_new_account_ignores_memo() {
        let fee = calculate_transaction_fee(&create_input(AssetType::JETTON, Some("memo".to_string())), None);
        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_CREATION)));
    }

    #[test]
    fn test_swap_contract_native_fee_includes_native_reserve() {
        let swap_data = SwapData::mock_contract(SwapProvider::StonfiV2, "400000000", "1000000", "710000000");
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap(Asset::from_chain(Chain::Ton), Asset::from_chain(Chain::Ton), swap_data),
            value: "400000000".to_string(),
            ..create_input(AssetType::NATIVE, None)
        };

        let fee = calculate_transaction_fee(&input, None);

        assert_eq!(fee.fee, BigInt::from(320000000u64));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(310000000u64)));
    }

    #[test]
    fn test_swap_contract_jetton_fee_includes_native_reserve() {
        let from_asset = Asset::mock_ton_usdt();
        let swap_data = SwapData::mock_contract(SwapProvider::StonfiV2, "2000000", "400000000", "300000000");
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap(from_asset, Asset::from_chain(Chain::Ton), swap_data),
            value: "2000000".to_string(),
            ..create_input(AssetType::JETTON, None)
        };

        let fee = calculate_transaction_fee(&input, None);

        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + SWAP_NATIVE_RESERVE));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(SWAP_NATIVE_RESERVE)));
    }

    #[test]
    fn test_swap_transfer_native_fee_uses_transfer_fee() {
        let swap_data = SwapData::mock_transfer(SwapProvider::NearIntents, "400000000", "1000000", "ton_deposit_address");
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap(Asset::from_chain(Chain::Ton), Asset::from_chain(Chain::Near), swap_data),
            value: "400000000".to_string(),
            ..create_input(AssetType::NATIVE, None)
        };

        let fee = calculate_transaction_fee(&input, None);

        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE));
        assert_eq!(fee.options.len(), 0);
    }

    #[test]
    fn test_swap_transfer_jetton_fee_uses_token_transfer_fee() {
        let swap_data = SwapData::mock_transfer(SwapProvider::NearIntents, "2000000", "400000000", "ton_deposit_address");
        let input = TransactionLoadInput {
            input_type: TransactionInputType::Swap(Asset::mock_ton_usdt(), Asset::from_chain(Chain::Near), swap_data),
            value: "2000000".to_string(),
            ..create_input(AssetType::JETTON, None)
        };

        let fee = calculate_transaction_fee(&input, None);

        assert_eq!(fee.fee, BigInt::from(TON_BASE_FEE + JETTON_ACCOUNT_CREATION));
        assert_eq!(fee.options.get(&FeeOption::TokenAccountCreation), Some(&BigInt::from(JETTON_ACCOUNT_CREATION)));
    }

    #[test]
    fn test_get_recipient_jetton_wallet() {
        let transfer = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_ton_usdt()),
            sender_address: "sender".to_string(),
            destination_address: "recipient".to_string(),
        };
        assert_eq!(get_recipient_jetton_wallet(&transfer), Some("recipient"));

        let swap_data = SwapData::mock_transfer(SwapProvider::NearIntents, "2000000", "400000000", "ton_deposit_address");
        let transfer_swap = TransactionPreloadInput {
            input_type: TransactionInputType::Swap(Asset::mock_ton_usdt(), Asset::from_chain(Chain::Ethereum), swap_data),
            sender_address: "sender".to_string(),
            destination_address: "0xrecipient".to_string(),
        };
        assert_eq!(get_recipient_jetton_wallet(&transfer_swap), Some("ton_deposit_address"));

        let contract_swap = TransactionPreloadInput {
            input_type: TransactionInputType::Swap(
                Asset::mock_ton_usdt(),
                Asset::from_chain(Chain::Ton),
                SwapData::mock_contract(SwapProvider::StonfiV2, "2000000", "400000000", "300000000"),
            ),
            sender_address: "sender".to_string(),
            destination_address: "recipient".to_string(),
        };
        assert_eq!(get_recipient_jetton_wallet(&contract_swap), None);
    }
}
