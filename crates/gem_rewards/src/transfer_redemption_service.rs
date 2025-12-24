use crate::transfer_provider::{EvmClientProvider, EvmTransferProvider, RedemptionProvider, WalletConfig};
use crate::{RedemptionRequest, RedemptionResult, RedemptionService};
use primitives::ChainType;
use std::collections::HashMap;
use std::error::Error;

pub struct TransferRedemptionService {
    evm_provider: EvmTransferProvider,
}

impl TransferRedemptionService {
    pub fn new(wallets: HashMap<ChainType, WalletConfig>, client_provider: EvmClientProvider) -> Self {
        let evm_provider = EvmTransferProvider::new(wallets, client_provider);
        Self { evm_provider }
    }
}

impl RedemptionService for TransferRedemptionService {
    async fn process_redemption(&self, request: RedemptionRequest) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        self.evm_provider.process_redemption(request).await
    }
}
