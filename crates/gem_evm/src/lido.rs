use crate::erc2612::Permit;
use alloy_core::primitives::{Address, FixedBytes, U256};
use alloy_core::{sol, sol_types::SolCall};
use anyhow::Error;
use std::str::FromStr;

sol! {
    #[derive(Debug, PartialEq)]
    struct PermitInput {
        uint256 value;
        uint256 deadline;
        uint8 v;
        bytes32 r;
        bytes32 s;
    }

    #[derive(Debug, PartialEq)]
    interface WithdrawalQueueERC721 {
        function requestWithdrawalsWithPermit(uint256[] _amounts, address _owner, PermitInput _permit) returns (uint256[] requestIds);
        function claimWithdrawal(uint256 _requestId);
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface Lido {
        function submit(address _referral) payable returns (uint256);
    }
}

pub fn encode_submit_with_referral(referral: &str) -> Result<Vec<u8>, Error> {
    let _referral = if referral.is_empty() {
        Address::new([0u8; 20])
    } else {
        Address::from_str(referral).map_err(Error::msg)?
    };
    let call = Lido::submitCall { _referral };
    Ok(call.abi_encode())
}

pub fn encode_request_withdrawals_with_permit(
    amounts: Vec<String>,
    owner: &str,
    permit: &Permit,
) -> Result<Vec<u8>, Error> {
    let mut _amounts = vec![];
    for amount in amounts.iter() {
        let uint256 = U256::from_str(amount).map_err(Error::msg)?;
        _amounts.push(uint256);
    }

    let r: [u8; 32] = permit
        .r
        .clone()
        .try_into()
        .map_err(|e| anyhow::anyhow!("invalid r in signature {:?}", e))?;
    let s: [u8; 32] = permit
        .s
        .clone()
        .try_into()
        .map_err(|e| anyhow::anyhow!("invalid s in signature {:?}", e))?;

    let call = WithdrawalQueueERC721::requestWithdrawalsWithPermitCall {
        _amounts,
        _owner: Address::from_str(owner).map_err(Error::msg)?,
        _permit: PermitInput {
            value: U256::from_str(&permit.value).map_err(Error::msg)?,
            deadline: U256::from_str(&permit.deadline).map_err(Error::msg)?,
            v: permit.v,
            r: FixedBytes::from_slice(&r),
            s: FixedBytes::from_slice(&s),
        },
    };
    Ok(call.abi_encode())
}

pub fn encode_claim_withdrawal(request_id: &str) -> Result<Vec<u8>, Error> {
    let request_id = U256::from_str(request_id).map_err(Error::msg)?;
    let call = WithdrawalQueueERC721::claimWithdrawalCall {
        _requestId: request_id,
    };
    Ok(call.abi_encode())
}

pub fn decode_request_withdrawals_return(result: &[u8]) -> Result<Vec<String>, Error> {
    let decoded =
        WithdrawalQueueERC721::requestWithdrawalsWithPermitCall::abi_decode_returns(result, true)
            .map_err(Error::msg)?;
    Ok(decoded
        .requestIds
        .into_iter()
        .map(|x| x.to_string())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;

    #[test]
    fn test_encode_submit_with_referral() -> Result<(), Error> {
        let referral = "0x4C49d4Bd6a571827B4A556a0e1e3071DA6231B9D";
        let result = encode_submit_with_referral(referral)?;

        assert_eq!(
            result,
            hex::decode("a1903eab0000000000000000000000004c49d4bd6a571827b4a556a0e1e3071da6231b9d")
                .unwrap()
        );

        let result = encode_submit_with_referral("")?;
        assert_eq!(
            result,
            hex::decode("a1903eab0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap()
        );

        Ok(())
    }
}
