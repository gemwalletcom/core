use primitives::EVMChain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct EVMChainConfig {
    pub min_priority_fee: u64,
    pub is_opstack: bool,
    pub rewards_percentiles: EVMHistoryRewardPercentiles,
}

type Int = u64;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct EVMHistoryRewardPercentiles {
    pub slow: Int,
    pub normal: Int,
    pub fast: Int,
}

impl EVMHistoryRewardPercentiles {
    pub fn all(&self) -> Vec<u64> {
        let mut all = vec![self.slow, self.normal, self.fast];
        all.sort();
        all
    }
}

pub fn get_evm_chain_config(chain: EVMChain) -> EVMChainConfig {
    EVMChainConfig {
        min_priority_fee: chain.min_priority_fee(),
        is_opstack: chain.is_opstack(),
        rewards_percentiles: EVMHistoryRewardPercentiles {
            slow: 25,
            normal: 50,
            fast: 75,
        },
    }
}
