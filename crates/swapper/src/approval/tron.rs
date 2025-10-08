use crate::{SwapperError, alien::RpcProvider, client_factory::create_tron_client, models::ApprovalType};
use alloy_primitives::U256;
use num_bigint::BigUint;
use primitives::swap::ApprovalData;
use std::sync::Arc;

pub async fn check_approval_tron(
    owner_address: &str,
    token_address: &str,
    spender_address: &str,
    amount: U256,
    provider: Arc<dyn RpcProvider>,
) -> Result<ApprovalType, SwapperError> {
    let client = create_tron_client(provider.clone()).map_err(|e| SwapperError::NetworkError(e.to_string()))?;
    let allowance = client
        .get_token_allowance(owner_address, token_address, spender_address)
        .await
        .map_err(|e| SwapperError::NetworkError(e.to_string()))?;
    let amount_big = BigUint::from_bytes_be(&amount.to_be_bytes::<32>());
    if allowance < amount_big {
        return Ok(ApprovalType::Approve(ApprovalData {
            token: token_address.to_string(),
            spender: spender_address.to_string(),
            value: amount.to_string(),
        }));
    }
    Ok(ApprovalType::None)
}
