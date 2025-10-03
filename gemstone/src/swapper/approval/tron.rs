use alloy_primitives::U256;
use std::sync::Arc;

use crate::{
    models::GemApprovalData,
    network::{AlienClient, AlienProvider},
    swapper::{SwapperError, models::ApprovalType},
};
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use primitives::Chain;

pub async fn check_approval_tron(
    owner_address: &str,
    token_address: &str,
    spender_address: &str,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
) -> Result<ApprovalType, SwapperError> {
    let endpoint = provider.get_endpoint(Chain::Tron).map_err(SwapperError::from)?;
    let base_client = AlienClient::new(endpoint, provider.clone());
    let trongrid_client = TronGridClient::new(base_client.clone(), String::new());
    let client = TronClient::new(base_client, trongrid_client);
    let allowance = client
        .get_token_allowance(owner_address, token_address, spender_address)
        .await
        .map_err(|e| SwapperError::NetworkError(e.to_string()))?;
    if allowance < amount {
        return Ok(ApprovalType::Approve(GemApprovalData {
            token: token_address.to_string(),
            spender: spender_address.to_string(),
            value: amount.to_string(),
        }));
    }
    Ok(ApprovalType::None)
}
