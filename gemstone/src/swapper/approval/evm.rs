use crate::network::{jsonrpc_client_with_chain, AlienProvider};
use crate::swapper::{eth_address, models::ApprovalType, Permit2ApprovalData, SwapperApprovalData, SwapperError};

use alloy_primitives::{hex::decode as HexDecode, Address, U256};
use alloy_sol_types::SolCall;

use gem_evm::{
    contracts::erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    permit2::IAllowanceTransfer,
};
use primitives::Chain;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum CheckApprovalType {
    ERC20 {
        owner: String,
        token: String,
        spender: String,
        amount: U256,
    },
    Permit2 {
        permit2_contract: String,
        owner: String,
        token: String,
        spender: String,
        amount: U256,
    },
}

pub async fn check_approval_erc20(
    owner: String,
    token: String,
    spender: String,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
    chain: &Chain,
) -> Result<ApprovalType, SwapperError> {
    let owner: Address = owner.as_str().parse().map_err(|_| SwapperError::InvalidAddress(owner))?;
    let spender: Address = spender.as_str().parse().map_err(|_| SwapperError::InvalidAddress(spender))?;
    let allowance_data = IERC20::allowanceCall { owner, spender }.abi_encode();
    let allowance_call = EthereumRpc::Call(TransactionObject::new_call(&token, allowance_data), BlockParameter::Latest);

    let client = jsonrpc_client_with_chain(provider.clone(), *chain);
    let result: String = client.request(allowance_call).await.map_err(SwapperError::from)?;
    let decoded = HexDecode(result).map_err(|_| SwapperError::ABIError("failed to decode allowance_call result".into()))?;

    let allowance = IERC20::allowanceCall::abi_decode_returns(&decoded).map_err(SwapperError::from)?;
    if allowance < amount {
        return Ok(ApprovalType::Approve(SwapperApprovalData {
            token: token.to_string(),
            spender: spender.to_string(),
            value: amount.to_string(),
        }));
    }
    Ok(ApprovalType::None)
}

pub async fn check_approval_permit2(
    permit2_contract: &str,
    owner: String,
    token: String,
    spender: String,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
    chain: &Chain,
) -> Result<ApprovalType, SwapperError> {
    // Check permit2 allowance, spender is universal router
    let permit2_data = IAllowanceTransfer::allowanceCall {
        _0: eth_address::parse_str(&owner)?,
        _1: eth_address::parse_str(&token)?,
        _2: eth_address::parse_str(&spender)?,
    }
    .abi_encode();
    let permit2_call = EthereumRpc::Call(TransactionObject::new_call(permit2_contract, permit2_data), BlockParameter::Latest);

    let result: String = jsonrpc_client_with_chain(provider.clone(), *chain)
        .request(permit2_call)
        .await
        .map_err(SwapperError::from)?;
    let decoded = HexDecode(result).unwrap();
    let allowance_return = IAllowanceTransfer::allowanceCall::abi_decode_returns(&decoded).map_err(SwapperError::from)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
    let expiration: u64 = allowance_return
        ._1
        .try_into()
        .map_err(|_| SwapperError::ABIError("failed to convert expiration to u64".into()))?;

    if U256::from(allowance_return._0) < amount || expiration < timestamp {
        return Ok(ApprovalType::Permit2(Permit2ApprovalData {
            token,
            spender,
            value: amount.to_string(),
            permit2_contract: permit2_contract.into(),
            permit2_nonce: allowance_return
                ._2
                .try_into()
                .map_err(|_| SwapperError::ABIError("failed to convert nonce to u64".into()))?,
        }));
    }

    Ok(ApprovalType::None)
}

#[allow(unused)]
pub async fn check_approval(check_type: CheckApprovalType, provider: Arc<dyn AlienProvider>, chain: &Chain) -> Result<ApprovalType, SwapperError> {
    match check_type {
        CheckApprovalType::ERC20 { owner, token, spender, amount } => check_approval_erc20(owner, token, spender, amount, provider, chain).await,
        CheckApprovalType::Permit2 {
            permit2_contract,
            owner,
            token,
            spender,
            amount,
        } => check_approval_permit2(&permit2_contract, owner, token, spender, amount, provider, chain).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::mock::{AlienProviderMock, MockFn};
    use std::time::Duration;

    #[tokio::test]
    async fn test_approval_tx_spender_is_permit2() -> Result<(), SwapperError> {
        // Replicate https://optimistic.etherscan.io/tx/0x6aaa37e0ffdfcf0a0a45236cd39eb25fa9f3787133b583feeacc5d633f3e92f1
        // Make sure use checksum addresses
        let token = "0xdC6fF44d5d932Cbd77B52E5612Ba0529DC6226F1".to_string(); // WLD
        let owner = "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string();
        let spender = "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8".to_string(); // Router
        let permit2_contract = "0x000000000022D473030F116dDEE9F6B43aC78BA3".to_string();
        let amount = U256::from(1000000000000000000u64);
        let chain: Chain = Chain::Optimism;

        let token_clone = token.clone();
        let mock = AlienProviderMock {
            response: MockFn(Box::new(move |target| {
                let body = target.body.unwrap();
                let json = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
                let params = json["params"].as_array().unwrap();
                let param = params[0].as_object().unwrap();
                let to = param["to"].as_str().unwrap();
                if to == token_clone {
                    return r#"{"id":1,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#.to_string();
                }
                r#"{"id":1,"jsonrpc":"2.0","result":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}"#
                    .to_string()
            })),
            timeout: Duration::from_millis(10),
        };
        let provider = Arc::new(mock);

        let erc20_check = CheckApprovalType::ERC20 {
            owner: owner.clone(),
            token: token.clone(),
            spender: permit2_contract.clone(),
            amount,
        };
        let permit2_check = CheckApprovalType::Permit2 {
            permit2_contract: permit2_contract.clone(),
            owner: owner.clone(),
            token: token.clone(),
            spender: spender.clone(),
            amount,
        };

        let approvals = futures::future::join_all(vec![
            check_approval(erc20_check, provider.clone(), &chain),
            check_approval(permit2_check, provider.clone(), &chain),
        ])
        .await;

        let result: Vec<ApprovalType> = approvals.into_iter().flatten().collect();

        assert_eq!(
            result,
            vec![
                ApprovalType::Approve(SwapperApprovalData {
                    token: token.clone(),
                    spender: permit2_contract.clone(),
                    value: amount.to_string()
                }),
                ApprovalType::Permit2(Permit2ApprovalData {
                    token: token.clone(),
                    spender: spender.clone(),
                    value: amount.to_string(),
                    permit2_contract,
                    permit2_nonce: 0,
                }),
            ]
        );
        Ok(())
    }
}
