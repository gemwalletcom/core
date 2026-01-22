use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Deserialize)]
pub struct YoApiResponse<T> {
    pub data: T,
    pub message: String,
    #[serde(rename = "statusCode")]
    pub status_code: u32,
}

#[derive(Debug, Deserialize)]
pub struct YoPerformanceData {
    pub realized: YoFormattedValue,
    pub unrealized: YoFormattedValue,
}

#[derive(Debug, Deserialize)]
pub struct YoFormattedValue {
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub raw: u64,
    pub formatted: String,
}

impl YoPerformanceData {
    pub fn total_rewards_raw(&self) -> u64 {
        self.realized.raw.saturating_add(self.unrealized.raw)
    }
}
