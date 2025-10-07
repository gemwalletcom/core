use super::broker::SolanaVaultSwapResponse;
use crate::{alien::RpcProvider, client_factory::create_client_with_chain};

use alloy_primitives::hex;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use gem_solana::{jsonrpc::SolanaRpc, models::LatestBlockhash};
use primitives::Chain;
use solana_primitives::{AccountMeta, InstructionBuilder, Pubkey, TransactionBuilder};
use std::{str::FromStr, sync::Arc};

pub async fn build_solana_tx(fee_payer: &str, response: &SolanaVaultSwapResponse, provider: Arc<dyn RpcProvider>) -> Result<String, String> {
    let fee_payer = Pubkey::from_str(fee_payer).map_err(|_| "Invalid fee payer".to_string())?;
    let program_id = Pubkey::from_str(response.program_id.as_str()).map_err(|_| "Invalid program ID".to_string())?;
    let data = hex::decode(response.data.as_str()).map_err(|_| "Invalid data".to_string())?;

    let rpc_client = create_client_with_chain(provider, Chain::Solana);
    let blockhash_response: LatestBlockhash = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
    let recent_blockhash = blockhash_response.value.blockhash;
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

    Ok(STANDARD.encode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        alien::mock::{MockFn, ProviderMock},
        chainflip::broker::SolanaVaultSwapResponse,
    };
    use gem_jsonrpc::types::JsonRpcResponse;
    use std::time::Duration;

    #[tokio::test]
    async fn test_build_solana_tx_with_mocked_blockhash() -> Result<(), String> {
        let wallet_address = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC";
        let blockhash_b58 = "BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF".to_string();
        let mock = ProviderMock {
            response: MockFn(Box::new(move |_| {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "value": {
                            "blockhash": blockhash_b58,
                            "lastValidBlockHeight": 342893948
                        }
                    },
                    "id": 1757035220
                })
                .to_string()
            })),
            timeout: Duration::from_millis(10),
        };

        let provider = Arc::new(mock);
        let response: JsonRpcResponse<SolanaVaultSwapResponse> =
            serde_json::from_str(include_str!("./test/chainflip_sol_arb_usdc_quote_data.json")).map_err(|e| e.to_string())?;

        let tx_b64 = build_solana_tx(wallet_address, &response.result, provider).await?;

        assert_eq!(
            tx_b64,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAMHhfupPuKcYE+oWKNRaIwBKQhB6vsZxjpwpHXTx7w7758q21EdC4D4NruUv9F26xeVqhYm0WXVWkSIjeQIxD3II9tUC6aOjrGBy017zEItREWS3QDEQI/vMhwSVTo/1e2664X/uFi6gx6sRwFnSAPu1ODmcAsu2sf8IuwYArWOf4gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIigdk5jnaVQcjb3Nozv0qlnESZ8J6eouD4cHFznUdrH/mnbBDL8THYGfUWCdASi1avvhnxRbvqBSGASZBJCzCac8CA/vjlRh67l6xlM0hAuQsp8uvbznxa/E9H2wqvhzgEGBgUBAAMCBLYBoyZc4vNpjcSAHSwEAAAAAAQAAAAUAAAAUUvLH5qruQTmEGvRBStm0nBtu7cHAAAAAGwAAAAACgAAAIX7qT7inGBPqFijUWiMASkIQer7GcY6cKR108e8O++fyqFFtvP91HjpJvpzAtB1MQAAAAAAAAAAAAAAAAAAAAAAAB6D0pctPco6Mw1gwnd+5bjSVoPGP6NZEWmFYJgw9CBUBQAEAC0RAAAAOJJMwzRWGJDjqBlAc1NxDgk="
        );

        Ok(())
    }
}
