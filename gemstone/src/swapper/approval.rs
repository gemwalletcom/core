use super::{models::ApprovalType, ApprovalData, Permit2ApprovalData, SwapperError};
use crate::network::{jsonrpc::*, AlienProvider};

use alloy_core::{
    hex::decode as HexDecode,
    primitives::{Address, AddressError, U256},
    sol_types::SolCall,
};

use gem_evm::{
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    permit2::IAllowanceTransfer,
};
use primitives::Chain;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub enum CheckApprovalType {
    #[allow(dead_code)]
    /* owner, token, spender, amount */
    ERC20(String, String, String, U256),
    /* permit2 contract, owner, token, spender, amount */
    Permit2(String, String, String, String, U256),
}

impl From<AddressError> for SwapperError {
    fn from(err: AddressError) -> Self {
        SwapperError::InvalidAddress { address: err.to_string() }
    }
}

pub async fn check_approval_erc20(
    owner: String,
    token: String,
    spender: String,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
    chain: &Chain,
) -> Result<ApprovalType, SwapperError> {
    let owner = Address::parse_checksummed(owner, None).map_err(SwapperError::from)?;
    let spender = Address::parse_checksummed(spender, None).map_err(SwapperError::from)?;
    let allowance_data = IERC20::allowanceCall { owner, spender }.abi_encode();
    let allowance_call = EthereumRpc::Call(TransactionObject::new_call(&token, allowance_data), BlockParameter::Latest);

    let response = jsonrpc_call(&allowance_call, provider.clone(), chain).await.map_err(SwapperError::from)?;
    let result: String = response.take().map_err(SwapperError::from)?;
    let decoded = HexDecode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;

    let allowance = IERC20::allowanceCall::abi_decode_returns(&decoded, false)
        .map_err(|_| SwapperError::ABIError {
            msg: "Invalid erc20 allowance response".into(),
        })?
        ._0;
    if allowance < amount {
        return Ok(ApprovalType::Approve(ApprovalData {
            token: token.to_string(),
            spender: spender.to_string(),
            value: amount.to_string(),
        }));
    }
    Ok(ApprovalType::None)
}

pub async fn check_approval(check_type: CheckApprovalType, provider: Arc<dyn AlienProvider>, chain: &Chain) -> Result<ApprovalType, SwapperError> {
    match check_type {
        CheckApprovalType::ERC20(owner, token, spender, amount) => check_approval_erc20(owner, token, spender, amount, provider, chain).await,
        CheckApprovalType::Permit2(permit2_contract, owner, token, spender, amount) => {
            // Check token allowance, spender is permit2
            let self_approval = check_approval_erc20(owner.clone(), token.clone(), spender.clone(), amount, provider.clone(), chain).await?;

            // Return self_approval if it's not None
            if matches!(self_approval, ApprovalType::Approve(_)) {
                return Ok(self_approval);
            }

            // Check permit2 allowance, spender is universal router
            let permit2_data = IAllowanceTransfer::allowanceCall {
                _0: Address::parse_checksummed(owner, None).map_err(SwapperError::from)?,
                _1: Address::parse_checksummed(token.clone(), None).map_err(SwapperError::from)?,
                _2: Address::parse_checksummed(spender.clone(), None).map_err(SwapperError::from)?,
            }
            .abi_encode();
            let permit2_call = EthereumRpc::Call(TransactionObject::new_call(&permit2_contract, permit2_data), BlockParameter::Latest);

            let response = jsonrpc_call(&permit2_call, provider.clone(), chain).await.map_err(SwapperError::from)?;
            let result: String = response.take().map_err(SwapperError::from)?;
            let decoded = HexDecode(result).unwrap();
            let allowance_return = IAllowanceTransfer::allowanceCall::abi_decode_returns(&decoded, false).map_err(|_| SwapperError::ABIError {
                msg: "Invalid permit2 allowance response".into(),
            })?;

            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
            let expiration: u64 = allowance_return._1.try_into().map_err(|_| SwapperError::ABIError {
                msg: "failed to convert expiration to u64".into(),
            })?;

            if U256::from(allowance_return._0) < amount || expiration < timestamp {
                return Ok(ApprovalType::Permit2(Permit2ApprovalData {
                    token,
                    spender,
                    value: amount.to_string(),
                    permit2_contract,
                    permit2_nonce: allowance_return._2.try_into().map_err(|_| SwapperError::ABIError {
                        msg: "failed to convert nonce to u64".into(),
                    })?,
                }));
            }

            Ok(ApprovalType::None)
        }
    }
}
