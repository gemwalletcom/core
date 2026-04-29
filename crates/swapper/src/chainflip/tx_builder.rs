use super::broker::SolanaVaultSwapResponse;
#[cfg(test)]
use crate::solana::gas_limit_from_transaction;
use crate::{alien::RpcProvider, client_factory::create_client_with_chain, solana};

use alloy_primitives::hex;
use gem_encoding::encode_base64;
use gem_solana::{jsonrpc::SolanaRpc, models::LatestBlockhash};
use primitives::Chain;
use solana_primitives::{AccountMeta, InstructionBuilder, Pubkey, TransactionBuilder, compute_budget::set_compute_unit_limit};
use std::{str::FromStr, sync::Arc};

pub async fn build_solana_tx(fee_payer: &str, response: &SolanaVaultSwapResponse, provider: Arc<dyn RpcProvider>) -> Result<String, String> {
    let fee_payer = Pubkey::from_str(fee_payer).map_err(|_| "Invalid fee payer".to_string())?;
    let program_id = Pubkey::from_str(response.program_id.as_str()).map_err(|_| "Invalid program ID".to_string())?;
    let data = hex::decode(response.data.as_str()).map_err(|_| "Invalid data".to_string())?;

    let rpc_client = create_client_with_chain(provider, Chain::Solana);
    let blockhash_response: LatestBlockhash = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
    let recent_blockhash = blockhash_response.value.blockhash;
    let blockhash = bs58::decode(recent_blockhash).into_vec().map_err(|_| "Failed to decode blockhash".to_string())?;

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
    transaction_builder.add_instruction(set_compute_unit_limit(solana::DEFAULT_SWAP_GAS_LIMIT));
    transaction_builder.add_instruction(instruction);

    let transaction = transaction_builder.build().map_err(|e| e.to_string())?;
    let bytes = transaction.serialize_legacy().map_err(|e| e.to_string())?;

    Ok(encode_base64(&bytes))
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
        let response: JsonRpcResponse<SolanaVaultSwapResponse> = serde_json::from_str(include_str!("./test/chainflip_sol_arb_usdc_quote_data.json")).map_err(|e| e.to_string())?;

        let tx_b64 = build_solana_tx(wallet_address, &response.result, provider).await?;

        assert_eq!(gas_limit_from_transaction(&tx_b64).map_err(|e| e.to_string())?, u64::from(solana::DEFAULT_SWAP_GAS_LIMIT));
        assert_eq!(
            tx_b64,
            "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAQIhfupPuKcYE+oWKNRaIwBKQhB6vsZxjpwpHXTx7w7758q21EdC4D4NruUv9F26xeVqhYm0WXVWkSIjeQIxD3II9tUC6aOjrGBy017zEItREWS3QDEQI/vMhwSVTo/1e2664X/uFi6gx6sRwFnSAPu1ODmcAsu2sf8IuwYArWOf4gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAiKB2TmOdpVByNvc2jO/SqWcRJnwnp6i4PhwcXOdR2sf+adsEMvxMdgZ9RYJ0BKLVq++GfFFu+oFIYBJkEkLMJpzwID++OVGHruXrGUzSEC5Cyny69vOfFr8T0fbCq+HOAgUABQKgaAYABwYGAQADAgS2AaMmXOLzaY3EgB0sBAAAAAAEAAAAFAAAAFFLyx+aq7kE5hBr0QUrZtJwbbu3BwAAAABsAAAAAAoAAACF+6k+4pxgT6hYo1FojAEpCEHq+xnGOnCkddPHvDvvn8qhRbbz/dR46Sb6cwLQdTEAAAAAAAAAAAAAAAAAAAAAAAAeg9KXLT3KOjMNYMJ3fuW40laDxj+jWRFphWCYMPQgVAUABAAtEQAAADiSTMM0VhiQ46gZQHNTcQ4J"
        );

        Ok(())
    }
}
