#[cfg(feature = "rpc")]
use crate::{SUI_COIN_TYPE, SuiClient, SuiError, models::Coin};
#[cfg(feature = "rpc")]
use futures::try_join;
#[cfg(feature = "rpc")]
use num_traits::ToPrimitive;
use sui_transaction_builder::ObjectInput;

#[derive(Clone)]
pub struct TransactionBuilderInput {
    pub sender: String,
    pub gas_price: u64,
    pub gas_budget: u64,
    pub gas_objects: Vec<ObjectInput>,
}

impl TransactionBuilderInput {
    pub fn new(sender: impl Into<String>, gas_price: u64, gas_budget: u64, gas_objects: Vec<ObjectInput>) -> Self {
        Self {
            sender: sender.into(),
            gas_price,
            gas_budget,
            gas_objects,
        }
    }

    pub fn with_gas_budget(&self, gas_budget: u64) -> Self {
        let mut input = self.clone();
        input.gas_budget = gas_budget;
        input
    }

    #[cfg(feature = "rpc")]
    pub async fn prefetch(client: &SuiClient, sender: &str, gas_budget: u64) -> Result<Self, SuiError> {
        let gas_price = async {
            client
                .get_gas_price()
                .await
                .map_err(SuiError::from_display)?
                .to_u64()
                .ok_or_else(|| SuiError::invalid_input("Sui gas price overflow"))
        };
        let gas_coins = async { client.get_coins(sender, SUI_COIN_TYPE).await.map(|owned| owned.coins).map_err(SuiError::from_display) };
        let (gas_price, gas_coins) = try_join!(gas_price, gas_coins)?;
        if gas_coins.is_empty() {
            return Err(SuiError::NoGasCoins);
        }
        let gas_objects = gas_coins.iter().map(Coin::to_input).collect();

        Ok(Self::new(sender, gas_price, gas_budget, gas_objects))
    }
}
