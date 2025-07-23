use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use anyhow::Error;
use std::str::FromStr;

pub const HUB_READER_ADDRESS: &str = "0x830295c0abe7358f7e24bc38408095621474280b";
pub const STAKE_HUB_ADDRESS: &str = "0x0000000000000000000000000000000000002002";

sol! {
    #[derive(Debug, PartialEq)]
    interface IHubReader {
        struct Validator {
            address operatorAddress;
            bool jailed;
            string moniker;
            uint64 commission;
            uint64 apy;
        }

        struct Delegation {
            address delegatorAddress;
            address validatorAddress;
            uint256 amount;
            uint256 shares;
        }

        struct Undelegation {
            address delegatorAddress;
            address validatorAddress;
            uint256 amount;
            uint256 shares;
            uint256 unlockTime;
        }

        function getValidators(uint16 offset, uint16 limit) external view returns (Validator[] memory);
        function getDelegations(address delegator, uint16 offset, uint16 limit) external view returns (Delegation[] memory);
        function getUndelegations(address delegator, uint16 offset, uint16 limit) external view returns (Undelegation[] memory);
    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IStakeHub {
        function delegate(address operatorAddress, bool delegateVotePower) external payable;
        function undelegate(address operatorAddress, uint256 shares) external;
        function redelegate(address srcValidator, address dstValidator, uint256 shares, bool delegateVotePower) external;
        function claim(address operatorAddress, uint256 requestNumber) external;
        function claimBatch(address[] calldata operatorAddresses,uint256[] calldata requestNumbers) external;
    }
}

pub struct BscValidator {
    pub operator_address: String,
    pub moniker: String,
    pub commission: u64,
    pub apy: u64,
    pub jailed: bool,
}

pub struct BscDelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: String,
    pub shares: String,
}

pub struct BscUndelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: String,
    pub shares: String,
    pub unlock_time: String,
}

pub fn encode_validators_call(offset: u16, limit: u16) -> Vec<u8> {
    let call = IHubReader::getValidatorsCall { offset, limit };
    call.abi_encode()
}

pub fn decode_validators_return(result: &[u8]) -> Result<Vec<BscValidator>, Error> {
    let decoded = IHubReader::getValidatorsCall::abi_decode_returns(result).map_err(Error::msg)?;
    let validators = decoded
        .iter()
        .map(|validator| BscValidator {
            operator_address: validator.operatorAddress.to_string(),
            moniker: validator.moniker.to_string(),
            commission: validator.commission,
            apy: validator.apy,
            jailed: validator.jailed,
        })
        .collect();
    Ok(validators)
}

pub fn encode_delegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, Error> {
    let delegator = Address::from_str(delegator).map_err(Error::msg)?;
    let call = IHubReader::getDelegationsCall { delegator, offset, limit };
    Ok(call.abi_encode())
}

pub fn decode_delegations_return(result: &[u8]) -> Result<Vec<BscDelegation>, Error> {
    let decoded = IHubReader::getDelegationsCall::abi_decode_returns(result).map_err(anyhow::Error::msg)?;
    let delegations = decoded
        .iter()
        .map(|delegation| BscDelegation {
            delegator_address: delegation.delegatorAddress.to_string(),
            validator_address: delegation.validatorAddress.to_string(),
            amount: delegation.amount.to_string(),
            shares: delegation.shares.to_string(),
        })
        .collect();
    Ok(delegations)
}

pub fn encode_undelegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, Error> {
    let delegator = Address::from_str(delegator).map_err(Error::msg)?;
    let call = IHubReader::getUndelegationsCall { delegator, offset, limit };
    Ok(call.abi_encode())
}

pub fn decode_undelegations_return(result: &[u8]) -> Result<Vec<BscUndelegation>, Error> {
    let decoded = IHubReader::getUndelegationsCall::abi_decode_returns(result).map_err(Error::msg)?;
    let undelegations = decoded
        .iter()
        .map(|undelegation| BscUndelegation {
            delegator_address: undelegation.delegatorAddress.to_string(),
            validator_address: undelegation.validatorAddress.to_string(),
            amount: undelegation.amount.to_string(),
            shares: undelegation.shares.to_string(),
            unlock_time: undelegation.unlockTime.to_string(),
        })
        .collect();
    Ok(undelegations)
}

pub fn encode_delegate_call(operator_address: &str, delegate_vote_power: bool) -> Result<Vec<u8>, Error> {
    let operator_address = Address::from_str(operator_address).map_err(Error::msg)?;
    let call = IStakeHub::delegateCall {
        operatorAddress: operator_address,
        delegateVotePower: delegate_vote_power,
    };
    Ok(call.abi_encode())
}

pub fn encode_undelegate_call(operator_address: &str, shares: &str) -> Result<Vec<u8>, Error> {
    let address = Address::from_str(operator_address).map_err(Error::msg)?;
    let amount = U256::from_str(shares).map_err(anyhow::Error::msg)?;
    let call = IStakeHub::undelegateCall {
        operatorAddress: address,
        shares: amount,
    };
    Ok(call.abi_encode())
}

pub fn encode_redelegate_call(src_validator: &str, dst_validator: &str, shares: &str, delegate_vote_power: bool) -> Result<Vec<u8>, Error> {
    let src_validator = Address::from_str(src_validator).map_err(Error::msg)?;
    let dst_validator = Address::from_str(dst_validator).map_err(Error::msg)?;
    let amount = U256::from_str(shares).map_err(Error::msg)?;
    let call = IStakeHub::redelegateCall {
        srcValidator: src_validator,
        dstValidator: dst_validator,
        shares: amount,
        delegateVotePower: delegate_vote_power,
    };
    Ok(call.abi_encode())
}

pub fn encode_claim_call(operator_address: &str, request_number: u64) -> Result<Vec<u8>, Error> {
    let operator_address = Address::from_str(operator_address).map_err(Error::msg)?;
    let call = IStakeHub::claimCall {
        operatorAddress: operator_address,
        requestNumber: U256::from(request_number),
    };
    Ok(call.abi_encode())
}

pub fn encode_claim_batch_call(operator_addresses: Vec<String>, request_numbers: Vec<u64>) -> Result<Vec<u8>, Error> {
    let operator_addresses = operator_addresses
        .iter()
        .map(|x| Address::from_str(x).map_err(Error::msg))
        .collect::<Result<Vec<Address>, Error>>()?;
    let request_numbers = request_numbers.iter().map(|x| U256::from(*x)).collect::<Vec<U256>>();
    let call = IStakeHub::claimBatchCall {
        operatorAddresses: operator_addresses,
        requestNumbers: request_numbers,
    };
    Ok(call.abi_encode())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_validators_return() {
        let result = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000003e000000000000000000000000000000000000000000000000000000000000004c000000000000000000000000000000000000000000000000000000000000005a00000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000076000000000000000000000000000000000000000000000000000000000000008400000000000000000000000000000000000000000000000000000000000000920000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc000000000000000000000000000000000000000000000000000000000000017400000000000000000000000000000000000000000000000000000000000000064c6567656e640000000000000000000000000000000000000000000000000000000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000c900000000000000000000000000000000000000000000000000000000000000084c6567656e644949000000000000000000000000000000000000000000000000000000000000000000000000f2b1d86dc7459887b1f7ce8d840db1d87613ce7f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000001d300000000000000000000000000000000000000000000000000000000000000094c6567656e644949490000000000000000000000000000000000000000000000000000000000000000000000eace91702b20bc6ee62034ec7f5162d9a94bfbe4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000004416e6b72000000000000000000000000000000000000000000000000000000000000000000000000000000005ce21461e6472914f5e4d5b296c72125f26ed462000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000008b00000000000000000000000000000000000000000000000000000000000000095472616e636865737300000000000000000000000000000000000000000000000000000000000000000000005c38ff8ca2b16099c086bf36546e99b13d152c4c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000057000000000000000000000000000000000000000000000000000000000000000954575374616b696e6700000000000000000000000000000000000000000000000000000000000000000000001ae5f5c3cb452e042b0b7b9dc60596c9cd84baf6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000007b000000000000000000000000000000000000000000000000000000000000000446756a6900000000000000000000000000000000000000000000000000000000000000000000000000000000b12e8137ef499a1d81552db11664a9e617fd350a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000054d617468570000000000000000000000000000000000000000000000000000000000000000000000000000004dc1bf52da103452097df48505a6d01020ffb22b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009a000000000000000000000000000000000000000000000000000000000000000744656669626974000000000000000000000000000000000000000000000000000000000000000000000000007d0f8a6d1c8fbf929dcf4847a31e30d14923fa31000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000084e6f64655265616c000000000000000000000000000000000000000000000000").unwrap();
        let validators = decode_validators_return(&result).unwrap();
        assert_eq!(validators.len(), 10);
        assert_eq!(validators[0].operator_address, "0x773760b0708a5Cc369c346993a0c225D8e4043B1");
        assert_eq!(validators[0].moniker, "Legend");
        assert_eq!(validators[0].commission, 700);
        assert_eq!(validators[0].apy, 372);
    }

    #[test]
    fn test_decode_delegations_return() {
        let result = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000de0b6b3b015a6430000000000000000000000000000000000000000000000000dd62dce1850f388000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e09ef1d9101a1740000000000000000000000000000000000000000000000000e028d70463b87f8").unwrap();
        let delegations = decode_delegations_return(&result).unwrap();
        assert_eq!(delegations.len(), 2);
        assert_eq!(delegations[1].delegator_address, "0xee448667ffc3D15ca023A6deEf2D0fAf084C0716");
        assert_eq!(delegations[1].validator_address, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A");
        assert_eq!(delegations[1].amount, "1011602501587280244");
        assert_eq!(delegations[1].shares, "1009524779838572536");
    }

    #[test]
    fn test_decode_undelegations_return() {
        let result = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a000000000000000000000000000000000000000000000000016345785d89ffff00000000000000000000000000000000000000000000000001628aab7a64b3dc00000000000000000000000000000000000000000000000000000000664e7431").unwrap();
        let undelegations = decode_undelegations_return(&result).unwrap();
        assert_eq!(undelegations.len(), 1);
        assert_eq!(undelegations[0].delegator_address, "0xee448667ffc3D15ca023A6deEf2D0fAf084C0716");
        assert_eq!(undelegations[0].validator_address, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A");
        assert_eq!(undelegations[0].amount, "99999999999999999");
        assert_eq!(undelegations[0].shares, "99794610853032924");
        assert_eq!(undelegations[0].unlock_time, "1716417585");
    }

    #[test]
    fn test_encode_delegatie_call() {
        let data = encode_delegate_call("0x773760b0708a5Cc369c346993a0c225D8e4043B1", false).unwrap();

        assert_eq!(
            hex::encode(data),
            "982ef0a7000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_encode_undelegatie_call() {
        let data = encode_undelegate_call("0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A", "99794610853032924").unwrap();

        assert_eq!(
            hex::encode(data),
            "4d99dd16000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a00000000000000000000000000000000000000000000000001628aab7a64b3dc"
        );
    }

    #[test]
    fn test_encode_redelegatie_call() {
        let data = encode_redelegate_call(
            "0x773760b0708a5Cc369c346993a0c225D8e4043B1",
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A",
            "1196258548170776928",
            false,
        )
        .unwrap();

        assert_eq!(hex::encode(data), "59491871000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b1000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000001099f6cfbf3e61600000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_encode_claim_batch_call() {
        let data = encode_claim_batch_call(vec!["0xE5572297718e1943A92BfEde2E67A060439e8EFd".to_string()], vec![0]).unwrap();

        assert_eq!(hex::encode(data), "d7c2dfc8000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000e5572297718e1943a92bfede2e67a060439e8efd00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000");
    }

}
