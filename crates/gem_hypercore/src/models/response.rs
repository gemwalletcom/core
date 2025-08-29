use serde::{Deserialize, Serialize};

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreErrorResponse {
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreStatusErrorResponse {
    pub status: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderResponse {
    pub status: String,
    pub response: Option<HypercoreOrderResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderResponseData {
    pub data: Option<HypercoreOrderData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderData {
    pub statuses: Option<Vec<HypercoreOrderStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderStatus {
    pub filled: Option<HypercoreOrderFilled>,
    pub resting: Option<HypercoreOrderResting>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderFilled {
    pub oid: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercoreOrderResting {
    pub oid: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionBroadcastResponse {
    OrderResponse(HypercoreOrderResponse),
    StatusErrorResponse(HypercoreStatusErrorResponse),
    SimpleResponse(HypercoreResponse),
    ErrorResponse(HypercoreErrorResponse),
}

#[derive(Debug)]
pub enum HyperCoreBroadcastResult {
    Success(String),
    Error(String),
}

impl TransactionBroadcastResponse {
    pub fn into_result(self, data: String) -> HyperCoreBroadcastResult {
        match self {
            TransactionBroadcastResponse::OrderResponse(order) => {
                if order.status == "ok" {
                    if let Some(status) = order.response.and_then(|r| r.data).and_then(|d| d.statuses).and_then(|s| s.first().cloned()) {
                        if let Some(error) = status.error {
                            return HyperCoreBroadcastResult::Error(error);
                        }
                        if let Some(filled) = status.filled {
                            return HyperCoreBroadcastResult::Success(filled.oid.to_string());
                        }
                        if let Some(resting) = status.resting {
                            return HyperCoreBroadcastResult::Success(resting.oid.to_string());
                        }
                    }
                    HyperCoreBroadcastResult::Success(data)
                } else {
                    HyperCoreBroadcastResult::Error("Order failed".to_string())
                }
            }
            TransactionBroadcastResponse::StatusErrorResponse(status_error) => {
                if status_error.status == "err" {
                    HyperCoreBroadcastResult::Error(status_error.response)
                } else {
                    HyperCoreBroadcastResult::Error(format!("Request failed with status: {}", status_error.status))
                }
            }
            TransactionBroadcastResponse::SimpleResponse(simple) => match simple.status.as_str() {
                "ok" => HyperCoreBroadcastResult::Success(data),
                _ => HyperCoreBroadcastResult::Error("Request failed".to_string()),
            },
            TransactionBroadcastResponse::ErrorResponse(error) => HyperCoreBroadcastResult::Error(error.response),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_broadcast_error() {
        let json = include_str!("../../tests/data/order_broadcast_error.json");
        let response: TransactionBroadcastResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response.into_result("test".to_string()), HyperCoreBroadcastResult::Error(_)));
    }

    #[test]
    fn test_order_broadcast_filled() {
        let json = include_str!("../../tests/data/order_broadcast_filled.json");
        let response: TransactionBroadcastResponse = serde_json::from_str(json).unwrap();
        match response.into_result("test".to_string()) {
            HyperCoreBroadcastResult::Success(oid) => assert_eq!(oid, "134896397196"),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_order_broadcast_resting() {
        let json = include_str!("../../tests/data/order_broadcast_resting.json");
        let response: TransactionBroadcastResponse = serde_json::from_str(json).unwrap();
        match response.into_result("test".to_string()) {
            HyperCoreBroadcastResult::Success(oid) => assert_eq!(oid, "789012"),
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_order_broadcast_simple_error() {
        let json = include_str!("../../tests/data/order_broadcast_simple_error.json");
        let response: TransactionBroadcastResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response.into_result("test".to_string()), HyperCoreBroadcastResult::Error(_)));
    }
}
