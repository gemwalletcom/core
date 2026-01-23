use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Deserialize)]
pub struct YoApiResponse<T> {
    #[serde(default)]
    pub data: T,
    #[serde(rename = "statusCode")]
    pub status_code: u32,
}

#[derive(Debug, Default, Deserialize)]
pub struct YoPerformanceData {
    #[serde(default)]
    pub realized: YoFormattedValue,
    #[serde(default)]
    pub unrealized: YoFormattedValue,
}

#[derive(Debug, Default, Deserialize)]
pub struct YoFormattedValue {
    #[serde(default, deserialize_with = "deserialize_u64_from_str_or_int")]
    pub raw: u64,
    #[serde(default)]
    pub formatted: String,
}

impl YoPerformanceData {
    pub fn total_rewards_raw(&self) -> u64 {
        self.realized.raw.saturating_add(self.unrealized.raw)
    }
}
