use serde::{Deserialize, Serialize};

use crate::models::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusErrorResponse {
    pub status: String,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub status: String,
    pub response: Option<OrderResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponseData {
    pub data: Option<OrderData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderData {
    pub statuses: Option<Vec<OrderStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatus {
    pub filled: Option<OrderFilled>,
    pub resting: Option<OrderResting>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFilled {
    pub oid: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResting {
    pub oid: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionBroadcastResponse {
    OrderResponse(OrderResponse),
    StatusErrorResponse(StatusErrorResponse),
    SimpleResponse(Response),
    ErrorResponse(ErrorResponse),
}

#[derive(Debug)]
pub enum BroadcastResult {
    Success(String),
    Error(String),
}

impl TransactionBroadcastResponse {
    pub fn into_result(self, action_id: Option<String>) -> BroadcastResult {
        match self {
            TransactionBroadcastResponse::OrderResponse(order) => {
                if order.status == "ok" {
                    if let Some(status) = order.response.and_then(|r| r.data).and_then(|d| d.statuses).and_then(|s| s.first().cloned()) {
                        if let Some(error) = status.error {
                            return BroadcastResult::Error(error);
                        }
                        if let Some(filled) = status.filled {
                            return BroadcastResult::Success(filled.oid.to_string());
                        }
                        if let Some(resting) = status.resting {
                            return BroadcastResult::Success(resting.oid.to_string());
                        }
                    }
                    match action_id {
                        Some(id) => BroadcastResult::Success(id),
                        None => BroadcastResult::Error("Failed to parse action id".to_string()),
                    }
                } else {
                    BroadcastResult::Error("Order failed".to_string())
                }
            }
            TransactionBroadcastResponse::StatusErrorResponse(status_error) => {
                if status_error.status == "err" {
                    BroadcastResult::Error(status_error.response)
                } else {
                    BroadcastResult::Error(format!("Request failed with status: {}", status_error.status))
                }
            }
            TransactionBroadcastResponse::SimpleResponse(simple) => match (simple.status.as_str(), action_id) {
                ("ok", Some(id)) => BroadcastResult::Success(id),
                ("ok", None) => BroadcastResult::Error("Failed to parse action id".to_string()),
                _ => BroadcastResult::Error("Request failed".to_string()),
            },
            TransactionBroadcastResponse::ErrorResponse(error) => BroadcastResult::Error(error.response),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_broadcast_error() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(include_str!("../../testdata/order_broadcast_error.json")).unwrap().into_result(None);
        let BroadcastResult::Error(_) = result else { panic!("Expected error") };
    }

    #[test]
    fn test_order_broadcast_filled() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(include_str!("../../testdata/order_broadcast_filled.json")).unwrap().into_result(None);
        let BroadcastResult::Success(oid) = result else { panic!("Expected success") };
        assert_eq!(oid, "134896397196");
    }

    #[test]
    fn test_order_broadcast_resting() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(include_str!("../../testdata/order_broadcast_resting.json")).unwrap().into_result(None);
        let BroadcastResult::Success(oid) = result else { panic!("Expected success") };
        assert_eq!(oid, "789012");
    }

    #[test]
    fn test_order_broadcast_simple_error() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(include_str!("../../testdata/order_broadcast_simple_error.json")).unwrap().into_result(None);
        let BroadcastResult::Error(_) = result else { panic!("Expected error") };
    }

    #[test]
    fn test_order_broadcast_fallback_uses_action_id() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(r#"{"status":"ok","response":{"type":"order"}}"#).unwrap().into_result(Some("action:123".to_string()));
        let BroadcastResult::Success(id) = result else { panic!("Expected success") };
        assert_eq!(id, "action:123");
    }

    #[test]
    fn test_simple_broadcast_uses_action_id() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(r#"{"status":"ok"}"#).unwrap().into_result(Some("action:456".to_string()));
        let BroadcastResult::Success(id) = result else { panic!("Expected success") };
        assert_eq!(id, "action:456");
    }

    #[test]
    fn test_order_broadcast_fallback_without_action_id() {
        let result = serde_json::from_str::<TransactionBroadcastResponse>(r#"{"status":"ok","response":{"type":"order"}}"#).unwrap().into_result(None);
        let BroadcastResult::Error(_) = result else { panic!("Expected error") };
    }
}
