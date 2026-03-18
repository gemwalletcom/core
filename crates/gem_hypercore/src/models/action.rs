use serde::{Deserialize, Serialize};

pub const ACTION_ID_PREFIX: &str = "action:";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRequest {
    pub action: ExchangeAction,
    pub nonce: u64,
}

impl ExchangeRequest {
    pub fn get_nonce(data: &str) -> Option<u64> {
        if let Some(value) = data.strip_prefix(ACTION_ID_PREFIX) {
            return value.parse().ok();
        }
        serde_json::from_str::<ExchangeRequest>(data).ok().map(|r| r.nonce)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAction {
    #[serde(rename = "type")]
    pub action_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nonce_from_action_id() {
        assert_eq!(ExchangeRequest::get_nonce("action:1755132472149"), Some(1755132472149));
    }

    #[test]
    fn test_get_nonce_from_json() {
        let request = include_str!("../../testdata/hl_action_update_position_tp_sl.json").trim();
        assert_eq!(ExchangeRequest::get_nonce(request), Some(1755132472149));
    }

    #[test]
    fn test_get_nonce_invalid() {
        assert_eq!(ExchangeRequest::get_nonce("not-json"), None);
    }
}
