use primitives::Transaction;

use crate::SwapperProvider;
use crate::cross_chain::CrossChainProvider;

pub struct NearIntentsCrossChain;

impl CrossChainProvider for NearIntentsCrossChain {
    fn provider(&self) -> SwapperProvider {
        SwapperProvider::NearIntents
    }

    fn is_swap(&self, _transaction: &Transaction) -> bool {
        false
    }
}
