use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::constants::{AGGREGATOR_V3_PACKAGE, DEFAULT_AGGREGATOR_V3};

#[derive(Debug, Clone, Serialize)]
pub struct RouterRequest {
    pub from: String,
    pub target: String,
    pub amount: String,
    pub by_amount_in: bool,
    pub providers: String,
    pub v: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RouterResponse {
    Ok { data: RouterData },
    Err { code: u32, msg: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouterData {
    pub request_id: String,
    pub amount_out: u64,
    pub paths: Vec<Path>,
    pub packages: Option<BTreeMap<String, String>>,
}

impl RouterData {
    pub fn aggregator_v3(&self) -> String {
        self.packages
            .as_ref()
            .and_then(|packages| packages.get(AGGREGATOR_V3_PACKAGE))
            .cloned()
            .unwrap_or_else(|| DEFAULT_AGGREGATOR_V3.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Path {
    pub id: String,
    pub direction: bool,
    pub provider: String,
    pub from: String,
    pub target: String,
    pub amount_in: u64,
    pub published_at: Option<String>,
    pub extended_details: Option<ExtendedDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtendedDetails {
    pub deepbookv3_need_add_deep_price_point: Option<bool>,
    pub deepbookv3_reference_pool_id: Option<String>,
    pub deepbookv3_reference_pool_base_type: Option<String>,
    pub deepbookv3_reference_pool_quote_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedPath {
    pub path: Path,
    pub is_last_use_of_intermediate_token: bool,
}

impl FlattenedPath {
    pub fn amount_in(&self) -> u64 {
        if self.is_last_use_of_intermediate_token { u64::MAX } else { self.path.amount_in }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessedRouterData {
    pub request_id: String,
    pub from_coin_type: String,
    pub target_coin_type: String,
    pub flattened_paths: Vec<FlattenedPath>,
}

impl TryFrom<&RouterData> for ProcessedRouterData {
    type Error = crate::SwapperError;

    fn try_from(router: &RouterData) -> Result<Self, Self::Error> {
        let first = router.paths.first().ok_or(crate::SwapperError::InvalidRoute)?;
        let last = router.paths.last().ok_or(crate::SwapperError::InvalidRoute)?;
        let mut flattened_paths: Vec<_> = router
            .paths
            .iter()
            .cloned()
            .map(|path| FlattenedPath {
                path,
                is_last_use_of_intermediate_token: false,
            })
            .collect();

        let mut seen_tokens = BTreeMap::new();
        for flattened_path in flattened_paths.iter_mut().rev() {
            if !seen_tokens.contains_key(&flattened_path.path.from) {
                seen_tokens.insert(flattened_path.path.from.clone(), true);
                flattened_path.is_last_use_of_intermediate_token = true;
            }
        }

        Ok(Self {
            request_id: router.request_id.clone(),
            from_coin_type: first.from.clone(),
            target_coin_type: last.target.clone(),
            flattened_paths,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn router_response() -> RouterResponse {
        serde_json::from_str(include_str!("testdata/router_response.json")).unwrap()
    }

    fn router_data() -> RouterData {
        match router_response() {
            RouterResponse::Ok { data } => data,
            RouterResponse::Err { .. } => panic!("Expected router response"),
        }
    }

    #[test]
    fn test_parse_router_response() {
        let data = router_data();

        assert_eq!(data.request_id, "quote-id");
        assert_eq!(data.amount_out, 1916345);
        assert_eq!(data.paths[0].amount_in, 300000000);
        assert_eq!(data.aggregator_v3(), "0xaggregator");
        let value: Value = serde_json::to_value(&data).unwrap();
        assert_eq!(value.get("request_id").and_then(Value::as_str), Some("quote-id"));
    }

    #[test]
    fn test_process_flattened_paths() {
        let data = router_data();
        let processed = ProcessedRouterData::try_from(&data).unwrap();

        assert_eq!(processed.from_coin_type, "0x2::sui::SUI");
        assert_eq!(processed.target_coin_type, "0xdba::usdc::USDC");
        assert_eq!(
            processed.flattened_paths.iter().map(|path| path.is_last_use_of_intermediate_token).collect::<Vec<_>>(),
            vec![false, true, true]
        );
    }

    #[test]
    fn test_parse_error_response() {
        assert_eq!(
            serde_json::from_str::<RouterResponse>(r#"{"code":5000,"msg":"Insufficient liquidity"}"#).unwrap(),
            RouterResponse::Err {
                code: 5000,
                msg: "Insufficient liquidity".to_string(),
            }
        );
    }
}
