use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{sol, SolCall};
use anyhow::Error;
use std::str::FromStr;

use crate::contracts::erc2612::Permit;

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
    struct WithdrawalRequestStatus {
        uint256 amountOfStETH;
        uint256 amountOfShares;
        address owner;
        uint256 timestamp;
        bool isFinalized;
        bool isClaimed;
    }

    #[derive(Debug, PartialEq)]
    interface WithdrawalQueueERC721 {
        function requestWithdrawalsWithPermit(uint256[] _amounts, address _owner, PermitInput _permit) returns (uint256[] requestIds);
        function getWithdrawalRequests(address _owner) view returns (uint256[] requestsIds);
        function getWithdrawalStatus(uint256[] _requestIds) view returns (WithdrawalRequestStatus[] statuses);
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

pub fn encode_request_withdrawals_with_permit(amounts: Vec<String>, owner: &str, permit: &Permit) -> Result<Vec<u8>, Error> {
    let mut _amounts = vec![];
    for amount in amounts.iter() {
        let uint256 = U256::from_str(amount).map_err(Error::msg)?;
        _amounts.push(uint256);
    }

    let r: [u8; 32] = permit.r.clone().try_into().map_err(|e| anyhow::anyhow!("invalid r in signature {:?}", e))?;
    let s: [u8; 32] = permit.s.clone().try_into().map_err(|e| anyhow::anyhow!("invalid s in signature {:?}", e))?;

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

pub fn encode_get_withdrawal_request_ids(owner: &str) -> Result<Vec<u8>, Error> {
    let call = WithdrawalQueueERC721::getWithdrawalRequestsCall {
        _owner: Address::from_str(owner).map_err(Error::msg)?,
    };
    Ok(call.abi_encode())
}

pub fn decode_get_withdrawal_request_ids(result: &[u8]) -> Result<Vec<String>, Error> {
    let decoded = WithdrawalQueueERC721::getWithdrawalRequestsCall::abi_decode_returns(result).map_err(Error::msg)?;
    let requests = decoded.into_iter().map(|x| x.to_string()).collect();
    Ok(requests)
}

pub fn encode_get_withdrawal_request_status(request_ids: &[String]) -> Result<Vec<u8>, Error> {
    let mut _request_ids = vec![];
    for request_id in request_ids.iter() {
        let uint256 = U256::from_str(request_id).map_err(Error::msg)?;
        _request_ids.push(uint256);
    }

    let call = WithdrawalQueueERC721::getWithdrawalStatusCall { _requestIds: _request_ids };
    Ok(call.abi_encode())
}

pub fn decode_get_withdrawal_request_status(result: &[u8]) -> Result<Vec<WithdrawalRequestStatus>, Error> {
    let decoded = WithdrawalQueueERC721::getWithdrawalStatusCall::abi_decode_returns(result).map_err(Error::msg)?;
    Ok(decoded.into_iter().collect())
}

pub fn encode_claim_withdrawal(request_id: &str) -> Result<Vec<u8>, Error> {
    let request_id = U256::from_str(request_id).map_err(Error::msg)?;
    let call = WithdrawalQueueERC721::claimWithdrawalCall { _requestId: request_id };
    Ok(call.abi_encode())
}

pub fn decode_request_withdrawals_return(result: &[u8]) -> Result<Vec<String>, Error> {
    let decoded = WithdrawalQueueERC721::requestWithdrawalsWithPermitCall::abi_decode_returns(result).map_err(Error::msg)?;
    Ok(decoded.into_iter().map(|x| x.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::erc2612::Permit;
    #[test]
    fn test_encode_submit_with_referral() {
        let referral = "0x4C49d4Bd6a571827B4A556a0e1e3071DA6231B9D";
        let result = encode_submit_with_referral(referral).unwrap();

        assert_eq!(hex::encode(result), "a1903eab0000000000000000000000004c49d4bd6a571827b4a556a0e1e3071da6231b9d");

        let result = encode_submit_with_referral("").unwrap();
        assert_eq!(hex::encode(result), "a1903eab0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_encode_request_withdrawals_with_permit() {
        // https://etherscan.io/tx/0x96920c52e2d3c6f50b99863f541541a4023e438afed873618b4aa73c25abbf9a

        let signature =
            hex::decode("189dada4c2af64022607fa643de95fd2503d46161e39a89df2dfffe0cded151e606e38989dd407af9c52a262ec6ca85398b2aa4ab5c7378ab2797a25818d50f51c")
                .unwrap();
        let permit = Permit {
            value: "101381038929079186".to_string(),
            deadline: "115792089237316195423570985008687907853269984665640564039457584007913129639935".to_string(),
            v: signature[64],
            r: signature[0..32].to_vec(),
            s: signature[32..64].to_vec(),
        };
        let result =
            encode_request_withdrawals_with_permit(vec!["101381038929079186".to_string()], "0x5014f5CF5f9F14033316c333245B66189A709537", &permit).unwrap();

        assert_eq!(
            hex::encode(result),
            "acf41e4d00000000000000000000000000000000000000000000000000000000000000e00000000000000000000000005014f5cf5f9f14033316c333245b66189a70953700000000000000000000000000000000000000000000000001682d848c53eb92ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001c189dada4c2af64022607fa643de95fd2503d46161e39a89df2dfffe0cded151e606e38989dd407af9c52a262ec6ca85398b2aa4ab5c7378ab2797a25818d50f5000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000001682d848c53eb92"
        );
    }

    #[test]
    fn test_encode_get_withdrawal_request_ids() {
        let result = encode_get_withdrawal_request_ids("0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7").unwrap();

        assert_eq!(hex::encode(result), "7d031b65000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7");
    }

    #[test]
    fn test_decode_get_withdrawal_request_ids() {
        let result = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000009f9c").unwrap();
        let ids = decode_get_withdrawal_request_ids(&result).unwrap();

        assert_eq!(ids, vec!["40860".to_string()]);
    }

    #[test]
    fn test_encode_get_withdrawal_request_status() {
        let result = encode_get_withdrawal_request_status(&["40860".to_string()]).unwrap();

        assert_eq!(
            hex::encode(result),
            "b8c4b85a000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000009f9c"
        );
    }

    #[test]
    fn test_decode_get_withdrawal_request_status() {
        let result = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000238e5aea4c9d23000000000000000000000000000000000000000000000000001e711f512aad49000000000000000000000000514bcb1f9aabb904e6106bd1052b66d2706dbbb7000000000000000000000000000000000000000000000000000000006656910b00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let requests = decode_get_withdrawal_request_status(&result).unwrap();

        assert_eq!(requests[0].amountOfStETH.to_string(), "10008145313963299");
        assert_eq!(requests[0].amountOfShares.to_string(), "8568628620995913");
        assert_eq!(requests[0].owner.to_string(), "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7");
        assert_eq!(requests[0].timestamp.to_string(), "1716949259");
        assert!(requests[0].isFinalized);
        assert!(!requests[0].isClaimed);
    }
}
