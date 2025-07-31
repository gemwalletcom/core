use chrono::DateTime;
use num_bigint::Sign;

use crate::{
    metaplex::metadata::Metadata,
    model::{BlockTransaction, BlockTransactions, Extension, Signature, TokenInfo},
    COMPUTE_BUDGET_PROGRAM_ID, JUPITER_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM,
};
use primitives::{Asset, AssetId, AssetType, Chain, SwapProvider, Transaction, TransactionState, TransactionSwapMetadata, TransactionType};

pub struct SolanaMapper;

impl SolanaMapper {
    const CHAIN: Chain = Chain::Solana;

    pub fn map_block_transactions(transactions: &BlockTransactions) -> Vec<primitives::Transaction> {
        transactions
            .transactions
            .iter()
            .filter_map(|transaction| Self::map_transaction(transaction, transactions.block_time))
            .collect()
    }

    pub fn map_signatures_transactions(transactions: Vec<BlockTransaction>, signatures: Vec<Signature>) -> Vec<primitives::Transaction> {
        transactions
            .iter()
            .zip(signatures.iter())
            .filter_map(|(transaction, signature)| Self::map_transaction(transaction, signature.block_time))
            .collect()
    }

    pub fn map_transaction(transaction: &BlockTransaction, block_time: i64) -> Option<primitives::Transaction> {
        let chain = Self::CHAIN;
        let account_keys = transaction.transaction.message.account_keys.clone();
        let signatures = transaction.transaction.signatures.clone();
        let hash = transaction.transaction.signatures.first()?.to_string();
        let fee = transaction.meta.fee;
        let state = TransactionState::Confirmed;
        let fee_asset_id = chain.as_asset_id();
        let created_at = DateTime::from_timestamp(block_time, 0)?;

        // only accept single signature transactions
        if signatures.len() != 1 {
            return None;
        }

        // system transfer
        if (account_keys.len() == 3) && account_keys.last()? == SYSTEM_PROGRAM_ID
            || (account_keys.len() == 4 && account_keys.last()? == SYSTEM_PROGRAM_ID && account_keys.contains(&COMPUTE_BUDGET_PROGRAM_ID.to_string()))
        {
            let from = account_keys.first()?.clone();
            let to = account_keys[1].clone();

            let value = transaction.meta.pre_balances[0] - transaction.meta.post_balances[0] - fee;

            let transaction = Transaction::new(
                hash,
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                fee.to_string(),
                fee_asset_id,
                value.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        let pre_token_balances = transaction.meta.pre_token_balances.clone();
        let post_token_balances = transaction.meta.post_token_balances.clone();

        // SPL token transfer
        if let Some(first_balance) = pre_token_balances.first() {
            let token_id = &first_balance.mint;
            if account_keys.contains(&TOKEN_PROGRAM.to_string())
                && (pre_token_balances.len() == 1 || pre_token_balances.len() == 2)
                && post_token_balances.len() == 2
                && pre_token_balances.iter().all(|b| &b.mint == token_id)
                && post_token_balances.iter().all(|b| &b.mint == token_id)
            {
                let asset_id = AssetId {
                    chain,
                    token_id: Some(token_id.clone()),
                };

                let sender_account_index: i64 = if transaction.meta.pre_token_balances.len() == 1 {
                    transaction.meta.pre_token_balances.first()?.account_index
                } else if pre_token_balances.first()?.get_amount() >= post_token_balances.first()?.get_amount() {
                    pre_token_balances.first()?.account_index
                } else {
                    post_token_balances.last()?.account_index
                };
                let recipient_account_index = post_token_balances.iter().find(|b| b.account_index != sender_account_index)?.account_index;

                let sender = transaction.meta.get_post_token_balance(sender_account_index)?;
                let recipient = transaction.meta.get_post_token_balance(recipient_account_index)?;
                let from_value = transaction.meta.get_pre_token_balance(sender_account_index)?.get_amount();
                let to_value = transaction.meta.get_post_token_balance(sender_account_index)?.get_amount();

                if to_value > from_value {
                    return None;
                }
                let value = from_value - to_value;

                let from = sender.owner.clone();
                let to = recipient.owner.clone();

                let transaction = Transaction::new(
                    hash,
                    asset_id,
                    from,
                    to,
                    None,
                    TransactionType::Transfer,
                    state,
                    fee.to_string(),
                    fee_asset_id,
                    value.to_string(),
                    None,
                    None,
                    created_at,
                );
                return Some(transaction);
            }
        }

        if account_keys.contains(&JUPITER_PROGRAM_ID.to_string()) {
            let sender = account_keys.first()?.clone();
            let balance_changes = transaction.get_balance_changes_by_owner(&sender);
            let token_balance_changes = transaction.meta.get_token_balance_changes_by_owner(&sender);

            let (from_asset, from_value, to_asset, to_value) = match token_balance_changes.as_slice() {
                [a] => {
                    let (from, to) = if a.amount.sign() == Sign::Plus {
                        (&balance_changes, a)
                    } else {
                        (a, &balance_changes)
                    };
                    (
                        from.asset_id.clone(),
                        from.amount.magnitude().clone(),
                        to.asset_id.clone(),
                        to.amount.magnitude().clone(),
                    )
                }
                [a, b] => {
                    let (from, to) = if a.amount.sign() == Sign::Plus { (b, a) } else { (a, b) };
                    (
                        from.asset_id.clone(),
                        from.amount.magnitude().clone(),
                        to.asset_id.clone(),
                        to.amount.magnitude().clone(),
                    )
                }
                _ => return None,
            };

            let swap = TransactionSwapMetadata {
                from_asset: from_asset.clone(),
                from_value: from_value.clone().to_string(),
                to_asset: to_asset.clone(),
                to_value: to_value.clone().to_string(),
                provider: Some(SwapProvider::Jupiter.id().to_owned()),
            };

            let transaction = Transaction::new(
                hash.clone(),
                swap.from_asset.clone(),
                sender.clone(),
                sender.clone(),
                Some(JUPITER_PROGRAM_ID.to_string()),
                TransactionType::Swap,
                state,
                fee.to_string(),
                chain.as_asset_id(),
                swap.from_value.clone().to_string(),
                None,
                serde_json::to_value(swap.clone()).ok(),
                created_at,
            );
            return Some(transaction);
        }

        None
    }

    // fn asset_id_from_program(chain: Chain, program_id: String) -> AssetId {
    //     if program_id == WSOL_TOKEN_ADDRESS {
    //         chain.as_asset_id()
    //     } else {
    //         AssetId {
    //             chain,
    //             token_id: Some(program_id),
    //         }
    //     }
    // }

    pub fn map_token_data(chain: Chain, token_id: String, token_info: &TokenInfo, meta: &Metadata) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        let name = meta.data.name.trim_matches(char::from(0)).to_string();
        let symbol = meta.data.symbol.trim_matches(char::from(0)).to_string();
        let decimals = token_info.decimals;

        Ok(Asset::new(AssetId::from_token(chain, &token_id), name, symbol, decimals, AssetType::TOKEN))
    }

    pub fn map_token_data_spl_token_2022(chain: Chain, token_id: String, token_info: &TokenInfo) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        let token_metadata = token_info
            .extensions
            .as_ref()
            .and_then(|extensions| {
                extensions.iter().find_map(|ext| {
                    if let Extension::TokenMetadata(token_metadata) = ext {
                        Some(token_metadata.state.clone())
                    } else {
                        None
                    }
                })
            })
            .ok_or("no token metadata found")?;
        Ok(Asset::new(
            AssetId::from_token(chain, &token_id),
            token_metadata.name,
            token_metadata.symbol,
            token_info.decimals,
            AssetType::TOKEN,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::ResultTokenInfo;

    use super::*;
    use primitives::JsonRpcResult;

    #[test]
    fn test_transaction_swap_token_to_sol() {
        let file = include_str!("../../testdata/swap_token_to_sol.json");
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(file).unwrap();

        let transaction = SolanaMapper::map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: AssetId::from_token(Chain::Solana, "BKpSnSdNdANUxKPsn4AQ8mf4b9BoeVs9JD1Q8cVkpump"),
            from_value: "393647577456".to_string(),
            to_asset: Chain::Solana.as_asset_id(),
            to_value: "139512057".to_string(),
            provider: Some(SwapProvider::Jupiter.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_token_to_token() {
        let file = include_str!("../../testdata/swap_token_to_token.json");
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(file).unwrap();

        let transaction = SolanaMapper::map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: AssetId::from_token(Chain::Solana, "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo"),
            from_value: "1000000".to_string(),
            to_asset: AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            to_value: "999932".to_string(),
            provider: Some(SwapProvider::Jupiter.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_sol_to_token() {
        let file = include_str!("../../testdata/swap_sol_to_token.json");
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(file).unwrap();

        let transaction = SolanaMapper::map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: Chain::Solana.as_asset_id(),
            from_value: "10000000".to_string(),
            to_asset: AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            to_value: "1678930".to_string(),
            provider: Some(SwapProvider::Jupiter.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_transfer_sol() {
        let file = include_str!("../../testdata/transfer_sol.json");
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(file).unwrap();

        let transaction = SolanaMapper::map_transaction(&result.result, 1751394455).unwrap();
        let expected = Transaction::new(
            "t6DpS6U7G2UG4QwDq4mPM7F45Rnttxp2pHGRwTYsF7frxAs7KmSWWDcpMneMUULbKndkZy8iUvSU1AZUsqzDCPN".to_string(),
            Chain::Solana.as_asset_id(),
            "DyB4TbDBqPUsCfsJMuoqjktEAod7D3KMNULSo7R1Rb61".to_string(),
            "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "5000".to_string(),
            Chain::Solana.as_asset_id(),
            "2173".to_string(),
            None,
            None,
            DateTime::from_timestamp(1751394455, 0).unwrap(),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_transaction_transfer_sol_with_compute() {
        let file = include_str!("../../testdata/transfer_sol_with_compute.json");
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(file).unwrap();

        let transaction = SolanaMapper::map_transaction(&result.result, 1750884182).unwrap();
        let expected = Transaction::new(
            "2QeBm7G7qLmVTCVKAkbSUuZvcFjg6mBRVqaVSKWSZsTqJxHVTzMUwxDtu1Myfu8RzpUv5YMEBFFpGbwVM9ZQY8DL".to_string(),
            Chain::Solana.as_asset_id(),
            "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H".to_string(),
            "7nVDzZUjrBA3gHs3gNcHidhmR96CH7KpKsU8pyBZGHUr".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "7500".to_string(),
            Chain::Solana.as_asset_id(),
            "69000000".to_string(),
            None,
            None,
            DateTime::from_timestamp(1750884182, 0).unwrap(),
        );

        assert_eq!(transaction, expected);
    }

    #[test]
    fn test_map_token_spl_token_2022() {
        let file = include_str!("../../testdata/pyusd_mint.json");
        let result = serde_json::from_str::<JsonRpcResult<ResultTokenInfo>>(file)
            .unwrap()
            .result
            .value
            .data
            .parsed
            .info;

        let token_data =
            SolanaMapper::map_token_data_spl_token_2022(Chain::Solana, "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo".to_string(), &result).unwrap();

        assert_eq!(token_data.name, "PayPal USD");
        assert_eq!(token_data.symbol, "PYUSD");
        assert_eq!(token_data.decimals, 6);
    }

    #[test]
    fn test_transaction_transfer_usdc() {
        let file = include_str!("../../testdata/usdc_transfer.json");
        let result: JsonRpcResult<crate::model::SingleTransaction> = serde_json::from_str(file).unwrap();

        let block_transaction = BlockTransaction {
            meta: result.result.meta,
            transaction: result.result.transaction,
        };

        let transaction = SolanaMapper::map_transaction(&block_transaction, result.result.block_time).unwrap();
        let expected = Transaction::new(
            "4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx".to_string(),
            AssetId::from_token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "37BenMAXFJMo3GaXKb2XLsNQXmd6VbbdShZWnwDj9D6k".to_string(),
            "3UJQqKq8Xyx4aVRmHgEwpQZiW7toYRQCTy6Bgp1RdKnK".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "5500".to_string(),
            Chain::Solana.as_asset_id(),
            "100000".to_string(),
            None,
            None,
            DateTime::from_timestamp(1753346616, 0).unwrap(),
        );

        assert_eq!(transaction, expected);
    }
}
