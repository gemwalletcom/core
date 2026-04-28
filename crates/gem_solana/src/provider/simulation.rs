use std::collections::HashSet;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainSimulation;
use gem_client::Client;
use gem_encoding::decode_base64;
use primitives::{SimulationInput, SimulationResult};
use solana_primitives::VersionedTransaction;

use crate::provider::simulation_mapper::map_simulation_result;
use crate::rpc::client::SolanaClient;

#[async_trait]
impl<C: Client + Clone> ChainSimulation for SolanaClient<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let bytes = decode_base64(&input.encoded_transaction).map_err(|_| "base64 decode failed")?;
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).map_err(|e| format!("parse transaction: {e}"))?;
        let account_keys: Vec<String> = transaction.account_keys().iter().map(|key| key.to_string()).collect();
        let signer_addresses: HashSet<String> = transaction
            .account_keys()
            .iter()
            .take(transaction.num_required_signatures() as usize)
            .map(|key| key.to_string())
            .collect();

        let simulation = self.simulate_encoded_transaction(&input.encoded_transaction).await?;
        Ok(map_simulation_result(&account_keys, &signer_addresses, simulation))
    }
}
