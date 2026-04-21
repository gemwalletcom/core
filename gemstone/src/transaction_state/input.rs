use primitives::{Chain, TransactionState, TransactionStateRequest, TransactionSwapMetadata};
use std::str::FromStr;
use swapper::SwapperProvider;

#[derive(uniffi::Record, Clone, Debug)]
pub struct TransactionStateInput {
    pub chain: Chain,
    pub hash: String,
    pub state: TransactionState,
    pub from: String,
    pub created_at_secs: i64,
    pub block_number: i64,
    pub swap_metadata: Option<TransactionSwapMetadata>,
}

impl TransactionStateInput {
    pub fn swap_provider(&self) -> Option<SwapperProvider> {
        self.swap_metadata.as_ref()?.provider.as_deref().and_then(|s| SwapperProvider::from_str(s).ok())
    }
}

impl From<&TransactionStateInput> for TransactionStateRequest {
    fn from(input: &TransactionStateInput) -> Self {
        Self {
            id: input.hash.clone(),
            sender_address: input.from.clone(),
            created_at: input.created_at_secs,
            block_number: input.block_number,
        }
    }
}
