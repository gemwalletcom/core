use alloy_core::{sol, sol_types::SolCall};

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
        }
        function getValidators(uint16 offset, uint16 limit) external view returns (Validator[] memory);
        function getDelegations(address delegator, uint16 offset, uint16 limit) external view returns (Delegation[] memory);
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
}

pub fn decode_validators_return(result: &[u8]) -> Result<Vec<BscValidator>, anyhow::Error> {
    let decoded = IHubReader::getValidatorsCall::abi_decode_returns(result, true)
        .map_err(anyhow::Error::msg)?
        ._0;
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

pub fn decode_delegations_return(result: &[u8]) -> Result<Vec<BscDelegation>, anyhow::Error> {
    let decoded = IHubReader::getDelegationsCall::abi_decode_returns(result, true)
        .map_err(anyhow::Error::msg)?
        ._0;
    let delegations = decoded
        .iter()
        .map(|delegation| BscDelegation {
            delegator_address: delegation.delegatorAddress.to_string(),
            validator_address: delegation.validatorAddress.to_string(),
            amount: delegation.amount.to_string(),
        })
        .collect();
    Ok(delegations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_validators_return() {
        let result = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000003e000000000000000000000000000000000000000000000000000000000000004c000000000000000000000000000000000000000000000000000000000000005a00000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000076000000000000000000000000000000000000000000000000000000000000008400000000000000000000000000000000000000000000000000000000000000920000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc000000000000000000000000000000000000000000000000000000000000017400000000000000000000000000000000000000000000000000000000000000064c6567656e640000000000000000000000000000000000000000000000000000000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000c900000000000000000000000000000000000000000000000000000000000000084c6567656e644949000000000000000000000000000000000000000000000000000000000000000000000000f2b1d86dc7459887b1f7ce8d840db1d87613ce7f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000001d300000000000000000000000000000000000000000000000000000000000000094c6567656e644949490000000000000000000000000000000000000000000000000000000000000000000000eace91702b20bc6ee62034ec7f5162d9a94bfbe4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000004416e6b72000000000000000000000000000000000000000000000000000000000000000000000000000000005ce21461e6472914f5e4d5b296c72125f26ed462000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000008b00000000000000000000000000000000000000000000000000000000000000095472616e636865737300000000000000000000000000000000000000000000000000000000000000000000005c38ff8ca2b16099c086bf36546e99b13d152c4c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000057000000000000000000000000000000000000000000000000000000000000000954575374616b696e6700000000000000000000000000000000000000000000000000000000000000000000001ae5f5c3cb452e042b0b7b9dc60596c9cd84baf6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000007b000000000000000000000000000000000000000000000000000000000000000446756a6900000000000000000000000000000000000000000000000000000000000000000000000000000000b12e8137ef499a1d81552db11664a9e617fd350a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000054d617468570000000000000000000000000000000000000000000000000000000000000000000000000000004dc1bf52da103452097df48505a6d01020ffb22b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009a000000000000000000000000000000000000000000000000000000000000000744656669626974000000000000000000000000000000000000000000000000000000000000000000000000007d0f8a6d1c8fbf929dcf4847a31e30d14923fa31000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000084e6f64655265616c000000000000000000000000000000000000000000000000").unwrap();
        let validators = decode_validators_return(&result).unwrap();
        assert_eq!(validators.len(), 10);
        assert_eq!(
            validators[0].operator_address,
            "0x773760b0708a5Cc369c346993a0c225D8e4043B1"
        );
        assert_eq!(validators[0].moniker, "Legend");
        assert_eq!(validators[0].commission, 700);
        assert_eq!(validators[0].apy, 372);
    }

    #[test]
    fn test_decode_delegations_return() {
        let result = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e0932bb88351eef").unwrap();
        let delegations = decode_delegations_return(&result).unwrap();
        assert_eq!(delegations.len(), 1);
        assert_eq!(
            delegations[0].delegator_address,
            "0xee448667ffc3D15ca023A6deEf2D0fAf084C0716"
        );
        assert_eq!(
            delegations[0].validator_address,
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A"
        );
        assert_eq!(delegations[0].amount, "1011395372346842863");
    }
}
