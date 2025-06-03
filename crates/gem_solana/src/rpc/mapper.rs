use chrono::Utc;
use num_bigint::Sign;
use primitives::SwapProvider;
use primitives::{chain::Chain, Asset, AssetId, AssetType, Transaction, TransactionState, TransactionSwapMetadata, TransactionType};

use crate::metaplex::metadata::Metadata;
use crate::model::{BlockTransaction, TokenInfo};
use crate::{JUPITER_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM};

pub struct SolanaMapper;

impl SolanaMapper {
    const CHAIN: Chain = Chain::Solana;

    pub fn map_transaction(transaction: &BlockTransaction, block_number: i64) -> Option<primitives::Transaction> {
        let chain = Self::CHAIN;
        let account_keys = transaction.transaction.message.account_keys.clone();
        let signatures = transaction.transaction.signatures.clone();
        let hash = transaction.transaction.signatures.first()?.to_string();
        let fee = transaction.meta.fee;
        let sequence = 0.to_string();
        let state = TransactionState::Confirmed;
        let fee_asset_id = chain.as_asset_id();
        let created_at = Utc::now();

        // system transfer
        if (account_keys.len() == 2 || account_keys.len() == 3) && account_keys.last()? == SYSTEM_PROGRAM_ID && signatures.len() == 1 {
            let from = account_keys.first()?.clone();
            let to = account_keys[account_keys.len() - 2].clone();

            let value = transaction.meta.pre_balances[0] - transaction.meta.post_balances[0] - fee;

            let transaction = Transaction::new(
                hash,
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                sequence,
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

        // SPL transfer. Limit to 7 accounts.
        if account_keys.contains(&TOKEN_PROGRAM.to_string())
            && account_keys.len() <= 7
            && (pre_token_balances.len() == 1 || pre_token_balances.len() == 2)
            && post_token_balances.len() == 2
        {
            let token_id = transaction.meta.pre_token_balances.first()?.mint.clone();
            let asset_id = AssetId {
                chain,
                token_id: Some(token_id),
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
                block_number.to_string(),
                sequence,
                fee.to_string(),
                fee_asset_id,
                value.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
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
                block_number.to_string(),
                sequence,
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
}

#[cfg(test)]
mod tests {
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
}
