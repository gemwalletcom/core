use std::error::Error;

use chrono::DateTime;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, AssetBalance, AssetId, AssetType, Transaction, TransactionState, TransactionType, chain::Chain};

use crate::{
    XRP_DEFAULT_ASSET_DECIMALS, XRP_EPOCH_OFFSET_SECONDS,
    rpc::model::{AccountLedger, AccountLedgerTransaction, AccountObject, Amount, Ledger},
};

use super::model::Transaction as XrpTransaction;

const RESULT_SUCCESS: &str = "tesSUCCESS";
const TRANSACTION_TYPE_PAYMENT: &str = "Payment";

pub struct XRPMapper;

impl XRPMapper {
    pub fn map_block_transactions(chain: Chain, ledger: Ledger) -> Vec<Transaction> {
        ledger
            .transactions
            .into_iter()
            .flat_map(|x| Self::map_block_transaction(chain, x, ledger.close_time))
            .collect::<Vec<Transaction>>()
    }

    pub fn map_account_transactions(chain: Chain, ledger: AccountLedger) -> Vec<Transaction> {
        ledger
            .transactions
            .into_iter()
            .flat_map(|x| Self::map_account_transaction(chain, x))
            .collect::<Vec<Transaction>>()
    }

    fn map_transaction_common(
        chain: Chain,
        hash: String,
        account: Option<String>,
        destination: Option<String>,
        amount: Option<Amount>,
        destination_tag: Option<i64>,
        fee: Option<String>,
        transaction_type: String,
        meta_result: String,
        timestamp: i64,
    ) -> Option<Transaction> {
        if transaction_type == TRANSACTION_TYPE_PAYMENT {
            let memo = destination_tag.map(|x| x.to_string());
            let value = amount.clone()?.as_value_string()?;
            let token_id = amount?.token_id();
            let asset_id = AssetId::from(chain, token_id);
            let created_at = DateTime::from_timestamp(timestamp, 0)?;

            let state = if meta_result == RESULT_SUCCESS {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };

            return Some(Transaction::new(
                hash,
                asset_id.clone(),
                account.unwrap_or_default(),
                destination.unwrap_or_default(),
                None,
                TransactionType::Transfer,
                state,
                fee.unwrap_or_default(),
                chain.as_asset_id(),
                value,
                memo,
                None,
                created_at,
            ));
        }
        None
    }

    pub fn map_account_transaction(chain: Chain, transaction: AccountLedgerTransaction) -> Option<Transaction> {
        Self::map_transaction_common(
            chain,
            transaction.hash,
            transaction.tx_json.account,
            transaction.tx_json.destination,
            transaction.tx_json.amount,
            transaction.tx_json.destination_tag,
            transaction.tx_json.fee,
            transaction.tx_json.transaction_type,
            transaction.meta.result,
            XRP_EPOCH_OFFSET_SECONDS + transaction.tx_json.date,
        )
    }

    pub fn map_block_transaction(chain: Chain, transaction: XrpTransaction, close_time: i64) -> Option<Transaction> {
        Self::map_transaction_common(
            chain,
            transaction.hash,
            transaction.account,
            transaction.destination,
            transaction.amount,
            transaction.destination_tag,
            transaction.fee,
            transaction.transaction_type,
            transaction.meta.result,
            XRP_EPOCH_OFFSET_SECONDS + close_time,
        )
    }

    pub fn map_token_data(chain: Chain, account_objects: Vec<AccountObject>) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let account = account_objects.first().ok_or("No account objects found for token_id")?;
        let symbol = account.low_limit.symbol().ok_or("Invalid currency")?;
        let token_id = &account.low_limit.issuer;

        Ok(Asset::new(
            AssetId::from_token(chain, token_id),
            symbol.clone(),
            symbol.clone(),
            XRP_DEFAULT_ASSET_DECIMALS,
            AssetType::TOKEN,
        ))
    }

    pub fn map_token_balances(chain: Chain, assets: Vec<AccountObject>) -> Vec<AssetBalance> {
        assets
            .into_iter()
            .filter(|x| x.high_limit.currency.len() > 3)
            .flat_map(|x| {
                let asset_id = AssetId::from_token(chain, &x.high_limit.issuer);
                let value = BigNumberFormatter::value_from_amount(&x.balance.value, XRP_DEFAULT_ASSET_DECIMALS as u32)?;
                Some(AssetBalance::new(asset_id, value))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::chain::Chain;
    use serde_json::from_str;

    #[test]
    fn test_map_account_transactions() {
        let test_data = {
            let response: serde_json::Value = from_str(include_str!("../testdata/account_transactions.json")).unwrap();
            from_str::<AccountLedger>(&response["result"].to_string()).unwrap()
        };
        let transactions = XRPMapper::map_account_transactions(Chain::Xrp, test_data);

        let expected_tx = Transaction::new(
            "00778C36255A48E753E7CDD3B60243D551ACD4B6ABD6765E9011D28B7566FEAB".to_string(),
            Chain::Xrp.as_asset_id(),
            "rGBpbVC11etyeGpJCAPrfS1of7SrEM2Q2c".to_string(),
            "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "11".to_string(),
            Chain::Xrp.as_asset_id(),
            "1".to_string(),
            None,
            None,
            DateTime::from_timestamp(1749150631, 0).unwrap(),
        );

        assert_eq!(transactions.first().unwrap(), &expected_tx);
    }
}
