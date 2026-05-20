mod order;
mod payload;
mod transaction;

use crate::mayan::{client::MayanClient, model::MayanSwiftQuote};
use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData, client_factory::create_client_with_chain};
use futures::try_join;
use gem_client::Client;
use gem_solana::{SolanaAddress, SolanaClient, encode_v0_transaction};
use primitives::Chain;
use solana_primitives::compute_budget;
use std::{fmt::Debug, fmt::Display, sync::Arc};

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let transaction = transaction::build(client, quote, route).await?;
    let rpc_client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana));
    let lookup_tables = async { rpc_client.get_address_lookup_tables(transaction.lookup_table_addresses).await.map_err(solana_error) };
    let blockhash = async { rpc_client.get_latest_blockhash().await.map(|response| response.value.blockhash).map_err(SwapperError::from) };
    let (lookup_tables, blockhash) = try_join!(lookup_tables, blockhash)?;
    let data = encode_v0_transaction(
        SolanaAddress::parse(&quote.request.wallet_address).map_err(solana_error)?.into(),
        &blockhash,
        &transaction.instructions,
        &lookup_tables,
    )
    .map_err(solana_error)?;
    let gas_limit = compute_budget::get_compute_unit_limit(&transaction.instructions).map(|limit| limit.to_string());

    Ok(SwapperQuoteData::new_contract(String::new(), "0".to_string(), data, None, gas_limit))
}

pub(super) fn solana_error(err: impl Display) -> SwapperError {
    SwapperError::transaction_error(err)
}
