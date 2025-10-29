use alloy_primitives::hex::encode_prefixed as HexEncode;
use primitives::swap::ApprovalData;

use super::asset::THORChainAsset;
use crate::SwapperQuoteData;

pub fn map_quote_data(
    from_asset: &THORChainAsset,
    router_address: String,
    call_data: Vec<u8>,
    inbound_address: String,
    value: String,
    memo: String,
    approval: Option<ApprovalData>,
    gas_limit: Option<String>,
) -> SwapperQuoteData {
    if from_asset.use_evm_router() {
        SwapperQuoteData::new_contract(router_address, value, HexEncode(call_data), approval, gas_limit)
    } else if from_asset.chain.is_evm_chain() {
        SwapperQuoteData::new_contract(inbound_address, value, HexEncode(memo.as_bytes()), approval, gas_limit)
    } else {
        SwapperQuoteData::new_tranfer(inbound_address, value, Some(memo))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thorchain::chain::THORChainName;
    use primitives::Chain;

    fn asset(chain: Chain, token_id: Option<String>) -> THORChainAsset {
        THORChainAsset {
            chain: THORChainName::from_chain(&chain).unwrap(),
            symbol: "TEST".to_string(),
            token_id,
            decimals: 18,
        }
    }

    #[test]
    fn evm_router() {
        let result = map_quote_data(
            &asset(Chain::Ethereum, Some("0xtoken".to_string())),
            "0xrouter".to_string(),
            vec![1, 2, 3],
            "0xinbound".to_string(),
            "1000".to_string(),
            "memo".to_string(),
            None,
            None,
        );

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.data, "0x010203");
    }

    #[test]
    fn evm_native() {
        let result = map_quote_data(
            &asset(Chain::Ethereum, None),
            "0xrouter".to_string(),
            vec![],
            "0xinbound".to_string(),
            "1000".to_string(),
            "memo".to_string(),
            None,
            None,
        );

        assert_eq!(result.to, "0xinbound");
        assert_eq!(result.data, "0x6d656d6f");
    }

    #[test]
    fn non_evm() {
        let result = map_quote_data(
            &asset(Chain::Bitcoin, None),
            "".to_string(),
            vec![],
            "bc1q".to_string(),
            "1000".to_string(),
            "memo".to_string(),
            None,
            None,
        );

        assert_eq!(result.to, "bc1q");
        assert_eq!(result.memo, Some("memo".to_string()));
    }

    #[test]
    fn approval_and_gas_limit() {
        let approval = Some(ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xspender".to_string(),
            value: "2000".to_string(),
        });

        let result = map_quote_data(
            &asset(Chain::Ethereum, Some("0xtoken".to_string())),
            "0xrouter".to_string(),
            vec![],
            "0xinbound".to_string(),
            "1000".to_string(),
            "memo".to_string(),
            approval.clone(),
            Some("100000".to_string()),
        );

        assert_eq!(result.approval, approval);
        assert_eq!(result.gas_limit, Some("100000".to_string()));
    }
}
