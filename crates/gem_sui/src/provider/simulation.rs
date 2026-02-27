use std::collections::HashMap;

use async_trait::async_trait;
use chain_traits::ChainSimulation;
use gem_client::Client;
use primitives::{BalanceDiff, BalanceDiffMap, SimulationInput, SimulationResult};

use crate::gas_budget::GasBudgetCalculator;
use crate::models::coin::BalanceChange;
use crate::provider::transactions_mapper::map_asset_id;
use crate::rpc::client::SuiClient;

fn map_dry_run_balance_changes(balance_changes: &[BalanceChange]) -> BalanceDiffMap {
    let mut result: BalanceDiffMap = HashMap::new();

    for change in balance_changes {
        let owner = match change.owner.get_address_owner() {
            Some(addr) => addr,
            None => continue,
        };
        let asset_id = map_asset_id(&change.coin_type);
        result.entry(owner).or_default().push(BalanceDiff {
            asset_id,
            from_value: None,
            to_value: None,
            diff: change.amount.clone(),
        });
    }

    result
}

#[async_trait]
impl<C: Client + Clone> ChainSimulation for SuiClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.dry_run(input.encoded_transaction).await?;
        let success = result.effects.is_success();
        let error = result.effects.error();

        let units_consumed = Some(GasBudgetCalculator::total_gas(&result.effects.gas_used));

        let balance_changes = map_dry_run_balance_changes(&result.balance_changes);

        Ok(SimulationResult {
            success,
            error,
            logs: vec![],
            units_consumed,
            balance_changes,
        })
    }
}
