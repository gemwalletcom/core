use alloy_core::primitives::U256;
use alloy_core::{sol, sol_types::SolCall};

sol! {
    #[derive(Debug, PartialEq)]
    interface IHubReader {
        struct Validator {
            address operatorAddress;
            bool jailed;
            string moniker;
            uint64 commission;
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
    pub jailed: bool,
}

pub struct BscDelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: U256,
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
            amount: delegation.amount,
        })
        .collect();
    Ok(delegations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_validators_return() {
        let result = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000064c6567656e640000000000000000000000000000000000000000000000000000000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000084c6567656e644949000000000000000000000000000000000000000000000000000000000000000000000000f2b1d86dc7459887b1f7ce8d840db1d87613ce7f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000094c6567656e644949490000000000000000000000000000000000000000000000").unwrap();
        let validators = decode_validators_return(&result).unwrap();
        assert_eq!(validators.len(), 3);
        assert_eq!(
            validators[0].operator_address,
            "0x773760b0708a5Cc369c346993a0c225D8e4043B1"
        );
        assert_eq!(validators[0].moniker, "Legend");
        assert_eq!(validators[0].commission, 700);
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
        assert_eq!(delegations[0].amount.to_string(), "1011395372346842863");
    }
}
