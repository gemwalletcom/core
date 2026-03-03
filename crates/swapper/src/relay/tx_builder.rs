use std::str::FromStr;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use gem_solana::{
    jsonrpc::SolanaRpc,
    models::{
        LatestBlockhash,
        rpc::{AccountData, ValueResult},
    },
    transaction::{self, LookupTable},
};
use primitives::{Chain, SolanaInstruction};
use solana_primitives::Pubkey;

use crate::{SwapperError, alien::RpcProvider, client_factory::create_client_with_chain};

pub use transaction::{ensure_compute_unit_price, get_unit_limit};

pub async fn build_solana_tx(
    fee_payer: &str,
    instructions: &[SolanaInstruction],
    lookup_table_addresses: &[String],
    provider: Arc<dyn RpcProvider>,
) -> Result<String, SwapperError> {
    let fee_payer_pk = Pubkey::from_str(fee_payer)?;
    let parsed = transaction::parse_instructions(instructions)?;

    let (recent_blockhash, lookup_tables) = futures::try_join!(fetch_recent_blockhash(&provider), fetch_lookup_tables(&provider, lookup_table_addresses))?;

    Ok(transaction::build_v0_transaction(fee_payer_pk, &parsed, &lookup_tables, recent_blockhash)?)
}

async fn fetch_recent_blockhash(provider: &Arc<dyn RpcProvider>) -> Result<[u8; 32], SwapperError> {
    let client = create_client_with_chain(provider.clone(), Chain::Solana);
    let response: LatestBlockhash = client
        .request(SolanaRpc::GetLatestBlockhash)
        .await
        .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;
    let bytes = bs58::decode(&response.value.blockhash).into_vec().map_err(|_| SwapperError::InvalidRoute)?;
    bytes.try_into().map_err(|_| SwapperError::InvalidRoute)
}

async fn fetch_lookup_tables(provider: &Arc<dyn RpcProvider>, addresses: &[String]) -> Result<Vec<LookupTable>, SwapperError> {
    if addresses.is_empty() {
        return Ok(vec![]);
    }
    let client = create_client_with_chain(provider.clone(), Chain::Solana);
    let response: ValueResult<Vec<Option<AccountData>>> = client
        .request(SolanaRpc::GetMultipleAccounts(addresses.to_vec()))
        .await
        .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;

    addresses
        .iter()
        .zip(response.value.iter())
        .map(|(address, account)| {
            let account = account.as_ref().ok_or(SwapperError::InvalidRoute)?;
            let data = STANDARD
                .decode(account.data.first().ok_or(SwapperError::InvalidRoute)?)
                .map_err(|_| SwapperError::InvalidRoute)?;
            Ok(transaction::parse_lookup_table(address, &data)?)
        })
        .collect()
}
