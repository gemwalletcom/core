use serde::Deserialize;

pub const ACTION_ID_KEY: &str = "action";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRequest {
    pub action: ExchangeAction,
    pub nonce: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ExchangeAction {
    Order,
    CDeposit {
        wei: u64,
    },
    CWithdraw {
        wei: u64,
    },
    TokenDelegate {
        wei: u64,
        is_undelegate: bool,
    },
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_request_parses_nonce() {
        let request = include_str!("../../testdata/hl_action_update_position_tp_sl.json").trim();
        assert_eq!(serde_json::from_str::<ExchangeRequest>(request).unwrap().nonce, 1755132472149);
    }

    #[test]
    fn test_exchange_request_rejects_invalid_json() {
        assert!(serde_json::from_str::<ExchangeRequest>("not-json").is_err());
    }
}
