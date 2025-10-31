use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolCall;
use gem_evm::thorchain::contracts::RouterInterface;
use num_bigint::BigInt;
use primitives::swap::ApprovalData;

use super::{DEFAULT_DEPOSIT_GAS_LIMIT, asset::THORChainAsset, model::RouteData};
use crate::SwapperQuoteData;

pub fn map_quote_data(
    from_asset: &THORChainAsset,
    route_data: &RouteData,
    token_id: Option<String>,
    value: String,
    memo: String,
    approval: Option<ApprovalData>,
) -> SwapperQuoteData {
    let gas_limit = if approval.is_some() {
        Some(DEFAULT_DEPOSIT_GAS_LIMIT.to_string())
    } else {
        None
    };

    if from_asset.use_evm_router() {
        let router_address = route_data.router_address.clone().unwrap_or_default();
        let inbound_address = Address::from_str(&route_data.inbound_address).unwrap();
        let token_address = Address::from_str(&token_id.unwrap()).unwrap();
        let amount = U256::from_str(&value).unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400;
        let expiry = U256::from_str(timestamp.to_string().as_str()).unwrap();

        let call_data = RouterInterface::depositWithExpiryCall {
            inbound_address,
            token_address,
            amount,
            memo: memo.clone(),
            expiry,
        }
        .abi_encode();

        SwapperQuoteData::new_contract(router_address, BigInt::ZERO.to_string(), HexEncode(call_data), approval, gas_limit)
    } else if from_asset.chain.is_evm_chain() {
        SwapperQuoteData::new_contract(route_data.inbound_address.clone(), value, HexEncode(memo.as_bytes()), approval, gas_limit)
    } else {
        SwapperQuoteData::new_tranfer(route_data.inbound_address.clone(), value, Some(memo))
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

    fn route_data(router: Option<String>, inbound: &str) -> RouteData {
        RouteData {
            router_address: router,
            inbound_address: inbound.to_string(),
        }
    }

    #[test]
    fn evm_router() {
        let result = map_quote_data(
            &asset(Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string())),
            &route_data(
                Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string()),
                "0x1234567890123456789012345678901234567890",
            ),
            Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()),
            "1000000".to_string(),
            "memo".to_string(),
            None,
        );

        assert_eq!(result.to, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146");
        assert_eq!(result.value, "0");
        assert!(result.data.starts_with("0x"));
        assert_eq!(result.memo, None);
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn evm_native() {
        let result = map_quote_data(
            &asset(Chain::Ethereum, None),
            &route_data(Some("0xrouter".to_string()), "0xinbound"),
            None,
            "1000".to_string(),
            "memo".to_string(),
            None,
        );

        assert_eq!(result.to, "0xinbound");
        assert_eq!(result.value, "1000");
        assert_eq!(result.data, "0x6d656d6f");
        assert_eq!(result.memo, None);
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn non_evm() {
        let result = map_quote_data(
            &asset(Chain::Bitcoin, None),
            &route_data(None, "bc1q"),
            None,
            "1000".to_string(),
            "memo".to_string(),
            None,
        );

        assert_eq!(result.to, "bc1q");
        assert_eq!(result.value, "1000");
        assert_eq!(result.data, "");
        assert_eq!(result.memo, Some("memo".to_string()));
        assert_eq!(result.gas_limit, None);
    }

    #[test]
    fn evm_router_with_approval() {
        let approval = Some(ApprovalData {
            token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            spender: "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string(),
            value: "2000".to_string(),
        });

        let result = map_quote_data(
            &asset(Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string())),
            &route_data(
                Some("0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string()),
                "0x1234567890123456789012345678901234567890",
            ),
            Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()),
            "1000000".to_string(),
            "memo".to_string(),
            approval.clone(),
        );

        assert_eq!(result.to, "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146");
        assert_eq!(result.value, "0");
        assert_eq!(result.approval, approval);
        assert_eq!(result.gas_limit, Some("90000".to_string()));
    }

    #[test]
    fn evm_native_without_approval() {
        let result = map_quote_data(
            &asset(Chain::Ethereum, None),
            &route_data(Some("0xrouter".to_string()), "0xinbound"),
            None,
            "1000".to_string(),
            "memo".to_string(),
            None,
        );

        assert_eq!(result.to, "0xinbound");
        assert_eq!(result.value, "1000");
        assert_eq!(result.gas_limit, None);
    }
}
