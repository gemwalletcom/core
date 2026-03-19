use chrono::DateTime;
use num_bigint::Sign;

use crate::{
    COMPUTE_BUDGET_PROGRAM_ID, JUPITER_PROGRAM_ID, OKX_DEX_V2_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSTEM_PROGRAMS, TOKEN_PROGRAM,
    models::{BlockTransaction, BlockTransactions, Signature},
};
use primitives::{AssetId, Chain, SwapProvider, Transaction, TransactionState, TransactionSwapMetadata, TransactionType};

const CHAIN: Chain = Chain::Solana;
const SWAP_PROGRAMS: &[(SwapProvider, &str)] = &[(SwapProvider::Jupiter, JUPITER_PROGRAM_ID), (SwapProvider::Okx, OKX_DEX_V2_PROGRAM_ID)];

fn get_swap_provider(account_keys: &[String]) -> Option<(SwapProvider, &'static str)> {
    SWAP_PROGRAMS.iter().copied().find(|(_, program_id)| account_keys.iter().any(|key| key == program_id))
}

fn map_swap_metadata(transaction: &BlockTransaction, owner: &str, provider: SwapProvider) -> Option<TransactionSwapMetadata> {
    let balance_changes = transaction.get_balance_changes_by_owner(owner);
    let token_balance_changes = transaction.meta.get_token_balance_changes_by_owner(owner);

    let (from_asset, from_value, to_asset, to_value) = match token_balance_changes.as_slice() {
        [change] => {
            let (from, to) = match change.amount.sign() {
                Sign::Plus => (&balance_changes, change),
                Sign::Minus => (change, &balance_changes),
                Sign::NoSign => return None,
            };
            (from.asset_id.clone(), from.amount.magnitude().clone(), to.asset_id.clone(), to.amount.magnitude().clone())
        }
        [a, b] => {
            let (from, to) = match a.amount.sign() {
                Sign::Plus => (b, a),
                Sign::Minus => (a, b),
                Sign::NoSign => return None,
            };
            (from.asset_id.clone(), from.amount.magnitude().clone(), to.asset_id.clone(), to.amount.magnitude().clone())
        }
        _ => return None,
    };

    Some(TransactionSwapMetadata {
        from_asset,
        from_value: from_value.to_string(),
        to_asset,
        to_value: to_value.to_string(),
        provider: Some(provider.id().to_owned()),
    })
}

pub fn map_block_transactions(transactions: &BlockTransactions) -> Vec<primitives::Transaction> {
    transactions
        .transactions
        .iter()
        .filter_map(|transaction| map_transaction(transaction, transactions.block_time))
        .collect()
}

pub fn map_signatures_transactions(transactions: Vec<BlockTransaction>, signatures: Vec<Signature>) -> Vec<primitives::Transaction> {
    transactions
        .iter()
        .zip(signatures.iter())
        .filter_map(|(transaction, signature)| map_transaction(transaction, signature.block_time))
        .collect()
}

pub fn map_transaction(transaction: &BlockTransaction, block_time: i64) -> Option<primitives::Transaction> {
    // only accept single signature transactions
    if transaction.transaction.signatures.len() != 1 {
        return None;
    }

    let chain = CHAIN;
    let account_keys = transaction.transaction.message.account_keys.clone();
    let hash = transaction.transaction.signatures.first()?.to_string();
    let fee = transaction.meta.fee;
    let state = if transaction.meta.err.is_some() {
        TransactionState::Failed
    } else {
        TransactionState::Confirmed
    };
    let fee_asset_id = chain.as_asset_id();
    let created_at = DateTime::from_timestamp(block_time, 0)?;

    // system transfer
    if (account_keys.len() == 3) && account_keys.last()? == SYSTEM_PROGRAM_ID
        || (account_keys.len() == 4 && account_keys.last()? == SYSTEM_PROGRAM_ID && account_keys.contains(&COMPUTE_BUDGET_PROGRAM_ID.to_string()))
    {
        let from = account_keys.first()?.clone();
        let to = account_keys[1].clone();

        let value = transaction.get_balance_change(&from);

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

    if let Some((provider, program_id)) = get_swap_provider(&account_keys) {
        let sender = account_keys.first()?.clone();
        let swap = map_swap_metadata(transaction, &sender, provider)?;

        let transaction = Transaction::new(
            hash.clone(),
            swap.from_asset.clone(),
            sender.clone(),
            sender.clone(),
            Some(program_id.to_string()),
            TransactionType::Swap,
            state,
            fee.to_string(),
            chain.as_asset_id(),
            swap.from_value.clone(),
            None,
            serde_json::to_value(swap).ok(),
            created_at,
        );
        return Some(transaction);
    }

    // smart contract call
    let contract = transaction
        .transaction
        .message
        .instructions
        .iter()
        .map(|ix| &account_keys[ix.program_id_index])
        .find(|key| !SYSTEM_PROGRAMS.contains(&key.as_str()))?;
    let sender = account_keys.first()?.clone();
    let value = transaction.get_balance_change(&sender);

    Some(Transaction::new(
        hash,
        chain.as_asset_id(),
        sender.clone(),
        sender,
        Some(contract.to_string()),
        TransactionType::SmartContractCall,
        state,
        fee.to_string(),
        fee_asset_id,
        value.to_string(),
        None,
        None,
        created_at,
    ))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        PYUSD_TOKEN_MINT, USDT_TOKEN_MINT,
        models::{SingleTransaction, SolanaTransaction},
    };
    use gem_jsonrpc::types::JsonRpcErrorResponse;
    use primitives::{JsonRpcResult, asset_constants::SOLANA_USDC_ASSET_ID};

    #[test]
    fn test_transaction_swap_token_to_sol() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_token_to_sol.json")).unwrap();

        let transaction = map_transaction(&result.result, 1).unwrap();
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
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_token_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: AssetId::from_token(Chain::Solana, PYUSD_TOKEN_MINT),
            from_value: "1000000".to_string(),
            to_asset: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
            to_value: "999932".to_string(),
            provider: Some(SwapProvider::Jupiter.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_sol_to_token() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_sol_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: Chain::Solana.as_asset_id(),
            from_value: "10000000".to_string(),
            to_asset: AssetId::from_token(Chain::Solana, USDT_TOKEN_MINT),
            to_value: "1678930".to_string(),
            provider: Some(SwapProvider::Jupiter.id().to_owned()),
        };

        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_swap_okx_token_to_token() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/swap_okx_token_to_token.json")).unwrap();

        let transaction = map_transaction(&result.result, 1).unwrap();
        let expected = TransactionSwapMetadata {
            from_asset: SOLANA_USDC_ASSET_ID.clone(),
            from_value: "56061275".to_string(),
            to_asset: AssetId::from_token(Chain::Solana, "HmMubgKx91Tpq3jmfcKQwsv5HrErqnCTTRJMB6afFR2u"),
            to_value: "2190151370200".to_string(),
            provider: Some(SwapProvider::Okx.id().to_owned()),
        };

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.asset_id, SOLANA_USDC_ASSET_ID.clone());
        assert_eq!(transaction.contract, Some(OKX_DEX_V2_PROGRAM_ID.to_string()));
        assert_eq!(transaction.value, "56061275");
        assert_eq!(transaction.metadata, Some(serde_json::to_value(expected).unwrap()));
    }

    #[test]
    fn test_transaction_transfer_sol() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/transfer_sol.json")).unwrap();

        let transaction = map_transaction(&result.result, 1751394455).unwrap();
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
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/transfer_sol_with_compute.json")).unwrap();

        let transaction = map_transaction(&result.result, 1750884182).unwrap();
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
    fn test_transaction_transfer_usdc() {
        let result: JsonRpcResult<SingleTransaction> = serde_json::from_str(include_str!("../../testdata/usdc_transfer.json")).unwrap();

        let block_transaction = BlockTransaction {
            meta: result.result.meta,
            transaction: result.result.transaction,
        };

        let transaction = map_transaction(&block_transaction, result.result.block_time).unwrap();
        let expected = Transaction::new(
            "4dHnggcXjvmMJY2J6iGqse12PeCYQzuTySgwJa36K8MuntmwNrCNztvYRX5ZGpQXzKjaf7g5vaZM7LTuXLNbi2Zx".to_string(),
            SOLANA_USDC_ASSET_ID.clone(),
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

    #[test]
    fn test_get_transaction_status() {
        let result: JsonRpcResult<SolanaTransaction> = serde_json::from_str(include_str!("../../testdata/transaction_state_transfer_sol.json")).unwrap();
        let transaction = result.result;

        let state = if transaction.slot > 0 {
            if transaction.meta.err.is_some() {
                TransactionState::Failed
            } else {
                TransactionState::Confirmed
            }
        } else {
            TransactionState::Pending
        };

        assert_eq!(state, TransactionState::Confirmed);
        assert_eq!(transaction.slot, 361169359);
    }

    #[test]
    fn test_transaction_chainflip_vault_swap() {
        let result: JsonRpcResult<BlockTransaction> = serde_json::from_str(include_str!("../../testdata/chainflip_vault_swap.json")).unwrap();
        let transaction = map_transaction(&result.result, 1772283531).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.from, "CabroWmzUzcqqGvprUoC7RnJznuwX6qf5W1tSSaomri7");
        assert_eq!(transaction.contract, Some("J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT".to_string()));
        assert_eq!(transaction.value, "152686560");
    }

    #[test]
    fn test_transaction_broadcast_error() {
        let error_response: JsonRpcErrorResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_swap_error.json")).unwrap();

        assert_eq!(error_response.error.code, -32002);
        assert_eq!(
            error_response.error.message,
            "Transaction simulation failed: Error processing Instruction 3: custom program error: 0x1771"
        );
        assert_eq!(error_response.id, Some(1755839259));
    }
}
