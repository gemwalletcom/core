mod asset;
mod chain;
mod client;
mod model;
mod provider;

use chain::THORChainName;

#[derive(Debug, Default)]
pub struct ThorChain {}

const QUOTE_MINIMUM: i64 = 0;
const QUOTE_INTERVAL: i64 = 1;
const QUOTE_QUANTITY: i64 = 0;

impl ThorChain {
    fn data(&self, chain: THORChainName, memo: String) -> String {
        if chain.is_evm_chain() {
            return hex::encode(memo.as_bytes());
        }
        memo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data() {
        let thorchain = ThorChain::default();
        let memo = "test".to_string();

        let result = thorchain.data(THORChainName::Ethereum, memo.clone());
        assert_eq!(result, hex::encode(memo.as_bytes()));

        let result = thorchain.data(THORChainName::Bitcoin, memo.clone());
        assert_eq!(result, memo);
    }
}
