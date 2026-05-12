use super::{ObjectResolver, TransactionBuilderInput};
use crate::{SuiClient, SuiError, is_sui_coin, models::CoinAsset};
use futures::try_join;
use gem_client::Client;
use std::collections::HashMap;

pub struct PrefetchedTransactionData {
    pub transaction: TransactionBuilderInput,
    pub input_coins: Vec<CoinAsset>,
    pub output_coin: Option<CoinAsset>,
    pub resolver: ObjectResolver,
}

impl PrefetchedTransactionData {
    pub async fn prefetch<C: Client + Clone>(
        client: &SuiClient<C>,
        sender: &str,
        input_coin_type: &str,
        output_coin_type: &str,
        object_ids: Vec<String>,
        pinned: &HashMap<String, u64>,
        gas_budget: u64,
    ) -> Result<Self, SuiError> {
        let (transaction, input_coins, output_coin, resolver) = try_join!(
            TransactionBuilderInput::prefetch(client, sender, gas_budget),
            fetch_input_coins(client, sender, input_coin_type),
            fetch_output_coin(client, sender, output_coin_type),
            ObjectResolver::prefetch(client, object_ids, pinned),
        )?;

        Ok(Self {
            transaction,
            input_coins,
            output_coin,
            resolver,
        })
    }
}

async fn fetch_input_coins<C: Client + Clone>(client: &SuiClient<C>, owner: &str, coin_type: &str) -> Result<Vec<CoinAsset>, SuiError> {
    if is_sui_coin(coin_type) {
        Ok(Vec::new())
    } else {
        client.get_coin_assets_by_type(owner, coin_type).await.map_err(|err| SuiError::invalid_input(err.to_string()))
    }
}

async fn fetch_output_coin<C: Client + Clone>(client: &SuiClient<C>, owner: &str, coin_type: &str) -> Result<Option<CoinAsset>, SuiError> {
    if is_sui_coin(coin_type) {
        Ok(None)
    } else {
        Ok(client
            .get_coin_assets_by_type(owner, coin_type)
            .await
            .map_err(|err| SuiError::invalid_input(err.to_string()))?
            .into_iter()
            .next())
    }
}
