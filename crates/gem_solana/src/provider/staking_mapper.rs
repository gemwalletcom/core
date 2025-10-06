use crate::models::{EpochInfo, TokenAccountInfo, VoteAccount};
use chrono::Utc;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn map_staking_validators(vote_accounts: Vec<VoteAccount>, chain: Chain, network_apy: f64) -> Vec<DelegationValidator> {
    vote_accounts
        .into_iter()
        .map(|validator| {
            let commission_rate = validator.commission as f64 / 100.0;
            let is_active = true;
            let validator_apr = if is_active { network_apy - (network_apy * commission_rate) } else { 0.0 };

            DelegationValidator {
                chain,
                id: validator.vote_pubkey,
                name: String::new(),
                is_active,
                commission: validator.commission as f64,
                apr: validator_apr,
            }
        })
        .collect()
}

pub fn map_staking_delegations(stake_accounts: Vec<TokenAccountInfo>, epoch: EpochInfo, asset_id: AssetId) -> Vec<DelegationBase> {
    stake_accounts
        .into_iter()
        .filter_map(|account| {
            if let Some(stake_info) = &account.account.data.parsed.info.stake {
                let balance = BigUint::from(account.account.lamports);
                let validator_id = stake_info.delegation.voter.clone();

                let activation_epoch = stake_info.delegation.activation_epoch;
                let deactivation_epoch = stake_info.delegation.deactivation_epoch;

                let is_active = deactivation_epoch == u64::MAX;

                let state = if !is_active {
                    if deactivation_epoch == epoch.epoch {
                        DelegationState::Deactivating
                    } else if deactivation_epoch < epoch.epoch {
                        DelegationState::AwaitingWithdrawal
                    } else {
                        DelegationState::Active
                    }
                } else if activation_epoch == epoch.epoch {
                    DelegationState::Activating
                } else if activation_epoch <= epoch.epoch {
                    DelegationState::Active
                } else {
                    DelegationState::Pending
                };

                let completion_date = match state {
                    DelegationState::Activating | DelegationState::Deactivating => {
                        let remaining_slots = epoch.slots_in_epoch - (epoch.epoch % epoch.slots_in_epoch);
                        let completion_seconds = remaining_slots as f64 * 0.420;
                        let completion_time = Utc::now() + chrono::Duration::milliseconds(completion_seconds as i64 * 1000);
                        Some(completion_time)
                    }
                    _ => None,
                };

                let rewards = BigUint::from(0u32);

                return Some(DelegationBase {
                    asset_id: asset_id.clone(),
                    state,
                    balance,
                    shares: BigUint::from(0u32),
                    rewards,
                    completion_date,
                    delegation_id: account.pubkey.clone(),
                    validator_id,
                });
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EpochInfo, TokenAccountData, TokenAccountInfo, VoteAccount};
    use primitives::{AssetId, Chain, DelegationState};

    #[test]
    fn test_map_staking_validators() {
        let vote_accounts = vec![VoteAccount {
            vote_pubkey: "validator1".to_string(),
            node_pubkey: "node1".to_string(),
            commission: 5,
        }];

        let result = map_staking_validators(vote_accounts, Chain::Solana, 8.0);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "validator1");
        assert_eq!(result[0].commission, 5.0);
        assert_eq!(result[0].apr, 7.6);
    }

    #[test]
    fn test_map_staking_delegations() {
        let stake_accounts = vec![TokenAccountInfo {
            pubkey: "stake1".to_string(),
            account: TokenAccountData {
                data: crate::models::Parsed {
                    parsed: crate::models::Info {
                        info: crate::models::TokenAccountInfoData {
                            mint: None,
                            token_amount: None,
                            stake: Some(crate::models::StakeInfo {
                                delegation: crate::models::StakeDelegation {
                                    activation_epoch: 100,
                                    deactivation_epoch: 18446744073709551615,
                                    stake: "1000000".to_string(),
                                    voter: "validator1".to_string(),
                                },
                            }),
                        },
                    },
                },
                owner: "owner1".to_string(),
                lamports: 1000000,
            },
        }];

        let epoch = EpochInfo {
            epoch: 200,
            slot_index: 0,
            slots_in_epoch: 432000,
        };

        let result = map_staking_delegations(stake_accounts, epoch, AssetId::from_chain(Chain::Solana));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].validator_id, "validator1");
        assert_eq!(result[0].balance.to_string(), "1000000");
        assert!(matches!(result[0].state, DelegationState::Active));
    }
}
