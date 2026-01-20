mod evm;

use crate::{RedemptionRequest, RedemptionResult};
use std::error::Error;

pub use evm::{EvmClientProvider, EvmTransferProvider, WalletConfig};

pub trait RedemptionProvider: Send + Sync {
    fn process_redemption(&self, request: RedemptionRequest) -> impl std::future::Future<Output = Result<RedemptionResult, Box<dyn Error + Send + Sync>>> + Send;
}
