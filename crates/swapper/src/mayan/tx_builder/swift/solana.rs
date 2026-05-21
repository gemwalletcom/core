mod order;
mod payload;
mod transaction;

use crate::mayan::{client::MayanClient, model::MayanSwiftQuote, tx_builder::solana as solana_builder};
use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData};
use gem_client::Client;
use std::{fmt::Debug, sync::Arc};

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let transaction = transaction::build(client, quote, route).await?;
    solana_builder::build_quote_data(quote, transaction, rpc_provider).await
}
