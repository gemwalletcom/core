use crate::address::TronAddress;
use crate::models::WitnessesList;
use primitives::{Chain, DelegationValidator, StakeValidator};

const SYSTEM_UNSTAKING_VALIDATOR_ID: &str = "system";
const SYSTEM_UNSTAKING_VALIDATOR_NAME: &str = "Unstaking";

pub fn map_validators(witnesses: WitnessesList) -> Vec<StakeValidator> {
    witnesses.witnesses.into_iter().map(|x| StakeValidator::new(x.address, x.url)).collect()
}

pub fn map_staking_validators(witnesses: WitnessesList, apy: Option<f64>) -> Vec<DelegationValidator> {
    let default_apy = apy.unwrap_or(0.0);
    let mut validators: Vec<DelegationValidator> = witnesses
        .witnesses
        .into_iter()
        .filter_map(|witness| {
            Some(DelegationValidator {
                chain: Chain::Tron,
                id: TronAddress::from_hex(&witness.address)?,
                name: String::new(),
                is_active: witness.is_jobs.unwrap_or(false),
                commision: 0.0,
                apr: default_apy,
            })
        })
        .collect();

    validators.push(DelegationValidator {
        chain: Chain::Tron,
        id: SYSTEM_UNSTAKING_VALIDATOR_ID.to_string(),
        name: SYSTEM_UNSTAKING_VALIDATOR_NAME.to_string(),
        is_active: true,
        commision: 0.0,
        apr: default_apy,
    });

    validators
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WitnessAccount;

    fn create_mock_witnesses() -> WitnessesList {
        WitnessesList {
            witnesses: vec![
                WitnessAccount {
                    address: "4159f3440fd40722f716144e4490a4de162d3b3fcb".to_string(),
                    vote_count: Some(1000000),
                    url: "https://validator1.com".to_string(),
                    is_jobs: Some(true),
                },
                WitnessAccount {
                    address: "41357a7401a0f0c2d4a44a1881a0c622f15d986291".to_string(),
                    vote_count: Some(500000),
                    url: "https://validator2.com".to_string(),
                    is_jobs: Some(false),
                },
            ],
        }
    }

    #[test]
    fn test_map_staking_validators() {
        let witnesses = create_mock_witnesses();
        let validators = map_staking_validators(witnesses, Some(4.2));

        assert_eq!(validators.len(), 3);

        assert_eq!(validators[0].chain, Chain::Tron);
        assert_eq!(validators[0].id, "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        assert_eq!(validators[0].name, "");
        assert!(validators[0].is_active);
        assert_eq!(validators[0].commision, 0.0);
        assert_eq!(validators[0].apr, 4.2);

        assert_eq!(validators[1].id, "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1");
        assert!(!validators[1].is_active);

        assert_eq!(validators[2].id, SYSTEM_UNSTAKING_VALIDATOR_ID);
        assert_eq!(validators[2].name, SYSTEM_UNSTAKING_VALIDATOR_NAME);
        assert!(validators[2].is_active);
    }
}
