use chrono::Utc;
use primitives::{chain::Chain, AssetId, Transaction, TransactionState, TransactionSwapMetadata, TransactionType};

use gem_solana::{TOKEN_PROGRAM, WSOL_TOKEN_ADDRESS};

use super::model::{BlockTransaction, InstructionParsed};

pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

pub struct SolanaMapper;

impl SolanaMapper {
    pub fn map_transaction(chain: Chain, transaction: &BlockTransaction, block_number: i64) -> Option<Transaction> {
        let account_keys = transaction
            .transaction
            .message
            .account_keys
            .clone()
            .into_iter()
            .map(|x| x.pubkey)
            .collect::<Vec<String>>();
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
            for inner_transaction in transaction.meta.inner_instructions.clone() {
                let instructions = inner_transaction
                    .instructions
                    .clone()
                    .into_iter()
                    .flat_map(|x| {
                        if let Some(value) = x.parsed {
                            return Some(value);
                        }
                        None
                    })
                    .collect::<Vec<InstructionParsed>>();

                let transfer_instructions = instructions
                    .into_iter()
                    .filter(|x| x.instruction_type == "transferChecked")
                    .collect::<Vec<InstructionParsed>>();

                // 1 - input, 2 - referral, 3 destination
                if transfer_instructions.len() == 3 {
                    let input = transfer_instructions.first()?.clone();
                    let output = transfer_instructions.last()?.clone();

                    let from_address = input.info.authority?;
                    let to_address = from_address.clone();

                    let from_asset = Self::asset_id_from_program(chain, input.info.mint?);
                    let to_asset = Self::asset_id_from_program(chain, output.info.mint?);
                    let from_value = input.info.token_amount?.amount.to_string();
                    let to_value = output.info.token_amount?.amount.to_string();

                    let swap = TransactionSwapMetadata {
                        from_asset: from_asset.clone(),
                        from_value: from_value.clone(),
                        to_asset: to_asset.clone(),
                        to_value: to_value.clone(),
                        provider: None,
                    };
                    let asset_id = from_asset.clone();

                    let transaction = Transaction::new(
                        hash.clone(),
                        asset_id,
                        from_address,
                        to_address,
                        Some(JUPITER_PROGRAM_ID.to_string()),
                        TransactionType::Swap,
                        state,
                        block_number.to_string(),
                        sequence,
                        fee.to_string(),
                        chain.as_asset_id(),
                        from_value.clone().to_string(),
                        None,
                        serde_json::to_value(swap).ok(),
                        created_at,
                    );
                    return Some(transaction);
                }
            }
        }

        None
    }

    fn asset_id_from_program(chain: Chain, program_id: String) -> AssetId {
        if program_id == WSOL_TOKEN_ADDRESS {
            return chain.as_asset_id();
        }
        AssetId {
            chain,
            token_id: Some(program_id),
        }
    }
}
