use super::broker::SolanaVaultSwapResponse;
use alien_provider::{jsonrpc::JsonRpcClient, AlienProvider};
use alloy_primitives::hex;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gem_solana::{jsonrpc::SolanaRpc, model::LatestBlockhash};
use primitives::Chain;
use solana_primitives::{AccountMeta, InstructionBuilder, Pubkey, TransactionBuilder};
use std::{str::FromStr, sync::Arc};

pub async fn build_solana_tx(fee_payer: String, response: &SolanaVaultSwapResponse, provider: Arc<dyn AlienProvider>) -> Result<String, String> {
    let fee_payer = Pubkey::from_str(fee_payer.as_str()).map_err(|_| "Invalid fee payer".to_string())?;
    let program_id = Pubkey::from_str(response.program_id.as_str()).map_err(|_| "Invalid program ID".to_string())?;
    let data = hex::decode(response.data.as_str()).map_err(|_| "Invalid data".to_string())?;
    let rpc_client = JsonRpcClient::new_with_chain(provider, Chain::Solana);
    let recent_blockhash = rpc_client
        .call::<SolanaRpc, LatestBlockhash>(&SolanaRpc::GetLatestBlockhash)
        .await
        .map_err(|e| e.to_string())?
        .take()
        .map_err(|e| e.to_string())?
        .value
        .blockhash;

    let blockhash = bs58::decode(recent_blockhash)
        .into_vec()
        .map_err(|_| "Failed to decode blockhash".to_string())?;

    let blockhash_array: [u8; 32] = blockhash.try_into().map_err(|_| "Failed to convert blockhash to array".to_string())?;

    let mut instruction = InstructionBuilder::new(program_id).data(data).build();
    response.accounts.iter().for_each(|account| {
        instruction.accounts.push(AccountMeta {
            is_signer: account.is_signer,
            is_writable: account.is_writable,
            pubkey: Pubkey::from_str(account.pubkey.as_str()).unwrap(),
        });
    });
    let mut transaction_builder = TransactionBuilder::new(fee_payer, blockhash_array);
    transaction_builder.add_instruction(instruction);
    let transaction = transaction_builder.build().map_err(|e| e.to_string())?;
    let bytes = transaction.serialize_legacy().map_err(|e| e.to_string())?;
    let base64_bytes = STANDARD.encode(&bytes);
    Ok(base64_bytes)
}
