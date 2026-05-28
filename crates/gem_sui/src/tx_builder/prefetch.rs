use super::{ObjectResolver, TransactionBuilderInput};
use crate::{
    SuiClient, SuiError, is_sui_coin,
    models::{Coin, OwnedCoins},
};
use futures::try_join;
use std::collections::HashMap;

pub struct PrefetchedTransactionData {
    pub transaction: TransactionBuilderInput,
    pub input_coins: OwnedCoins<Coin>,
    pub output_coin: Option<Coin>,
    pub resolver: ObjectResolver,
}

impl PrefetchedTransactionData {
    pub async fn prefetch(
        client: &SuiClient,
        sender: &str,
        input_coin_type: &str,
        output_coin_type: Option<&str>,
        object_ids: Vec<String>,
        pinned: &HashMap<String, u64>,
        gas_budget: u64,
    ) -> Result<Self, SuiError> {
        let output_coins_fut = async {
            match output_coin_type {
                Some(coin_type) => get_user_coins(client, sender, coin_type).await,
                None => Ok(OwnedCoins::default()),
            }
        };
        let (transaction, input_coins, output_owned, resolver) = try_join!(
            TransactionBuilderInput::prefetch(client, sender, gas_budget),
            get_user_coins(client, sender, input_coin_type),
            output_coins_fut,
            ObjectResolver::prefetch(client, object_ids, pinned),
        )?;

        Ok(Self {
            transaction,
            input_coins,
            output_coin: output_owned.coins.into_iter().next(),
            resolver,
        })
    }
}

async fn get_user_coins(client: &SuiClient, owner: &str, coin_type: &str) -> Result<OwnedCoins<Coin>, SuiError> {
    if is_sui_coin(coin_type) {
        Ok(OwnedCoins::default())
    } else {
        client.get_coins(owner, coin_type).await.map_err(SuiError::from_display)
    }
}
