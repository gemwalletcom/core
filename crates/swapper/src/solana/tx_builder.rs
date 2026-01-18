use crate::{alien::RpcProvider, client_factory::create_client_with_chain};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use gem_solana::{jsonrpc::SolanaRpc, models::LatestBlockhash};
use primitives::Chain;
use solana_primitives::{
    TransactionBuilder,
    types::{Instruction, Pubkey},
};
use std::sync::Arc;

pub async fn build_base64_transaction(fee_payer: Pubkey, instructions: Vec<Instruction>, provider: Arc<dyn RpcProvider>) -> Result<String, String> {
    let rpc_client = create_client_with_chain(provider, Chain::Solana);
    let blockhash_response: LatestBlockhash = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
    let recent_blockhash = blockhash_response.value.blockhash;
    let blockhash = bs58::decode(recent_blockhash).into_vec().map_err(|_| "Failed to decode blockhash".to_string())?;
    let blockhash_array: [u8; 32] = blockhash.try_into().map_err(|_| "Failed to convert blockhash to array".to_string())?;

    let mut transaction_builder = TransactionBuilder::new(fee_payer, blockhash_array);
    for instruction in instructions {
        transaction_builder.add_instruction(instruction);
    }

    let transaction = transaction_builder.build().map_err(|e| e.to_string())?;
    let bytes = transaction.serialize_legacy().map_err(|e| e.to_string())?;

    Ok(STANDARD.encode(&bytes))
}
