use primitives::EVMChain;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct EVMChainConfig {
    pub min_priority_fee: u64,
    pub is_opstack: bool,
    pub swap_whitelist_contracts: Vec<String>,
}

pub fn get_evm_chain_config(chain: EVMChain) -> EVMChainConfig {
    EVMChainConfig {
        min_priority_fee: chain.min_priority_fee(),
        is_opstack: chain.is_opstack(),
        swap_whitelist_contracts: chain
            .swap_whitelist_contracts()
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
    }
}
