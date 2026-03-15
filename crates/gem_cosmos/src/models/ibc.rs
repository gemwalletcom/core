use super::Coin;
use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IbcTransferValue {
    pub source_port: String,
    pub source_channel: String,
    pub token: Coin,
    pub sender: String,
    pub receiver: String,
    #[serde(default, deserialize_with = "deserialize_u64_from_str_or_int")]
    pub timeout_timestamp: u64,
    #[serde(default)]
    pub memo: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_timeout_as_number() {
        let json = r#"{"sourcePort":"transfer","sourceChannel":"channel-0","token":{"denom":"uatom","amount":"1000000"},"sender":"cosmos1test","receiver":"osmo1test","timeoutTimestamp":1773382733549000000,"memo":"test"}"#;
        let v: IbcTransferValue = serde_json::from_str(json).unwrap();
        assert_eq!(v.timeout_timestamp, 1773382733549000000);
    }

    #[test]
    fn test_deserialize_timeout_as_string() {
        let json = r#"{"sourcePort":"transfer","sourceChannel":"channel-0","token":{"denom":"uatom","amount":"1000000"},"sender":"cosmos1test","receiver":"osmo1test","timeoutTimestamp":"1773382733549000000","memo":"test"}"#;
        let v: IbcTransferValue = serde_json::from_str(json).unwrap();
        assert_eq!(v.timeout_timestamp, 1773382733549000000);
    }
}
