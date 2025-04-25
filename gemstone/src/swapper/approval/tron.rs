use alloy_primitives::U256;
use std::sync::Arc;

use crate::{
    network::AlienProvider,
    swapper::{models::ApprovalType, ApprovalData, SwapperError},
    tron::client::TronClient,
};

pub async fn check_approval_tron(
    owner_address: &str,
    token_address: &str,
    spender_address: &str,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
) -> Result<ApprovalType, SwapperError> {
    let client = TronClient::new(provider.clone());
    let allowance = client.get_token_allowance(owner_address, token_address, spender_address).await?;
    if allowance < amount {
        return Ok(ApprovalType::Approve(ApprovalData {
            token: token_address.to_string(),
            spender: spender_address.to_string(),
            value: amount.to_string(),
        }));
    }
    Ok(ApprovalType::None)
}
