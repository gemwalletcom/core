#[cfg(feature = "rpc")]
use crate::{SUI_COIN_TYPE, SuiClient, SuiError, models::CoinAsset};
#[cfg(feature = "rpc")]
use gem_client::Client;
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
    pub async fn prefetch<C: Client + Clone>(client: &SuiClient<C>, sender: &str, gas_budget: u64) -> Result<Self, SuiError> {
        let gas_price = client
            .get_gas_price()
            .await
            .map_err(|err| SuiError::invalid_input(err.to_string()))?
            .to_u64()
            .ok_or_else(|| SuiError::invalid_input("Sui gas price overflow"))?;
        let gas_coins = client
            .get_coin_assets_by_type(sender, SUI_COIN_TYPE)
            .await
            .map_err(|err| SuiError::invalid_input(err.to_string()))?;
        if gas_coins.is_empty() {
            return Err(SuiError::NoGasCoins);
        }
        let gas_objects = gas_coins.iter().map(CoinAsset::to_input).collect();

        Ok(Self::new(sender, gas_price, gas_budget, gas_objects))
    }
}
