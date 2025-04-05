use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub data: T,
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FearGreedData {
    pub data_list: Vec<FearGreedItem>,
    pub historical_values: HistoricalValues<FearGreedItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FearGreedItem {
    pub score: u32,
    pub name: String,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalValues<T> {
    pub now: T,
    pub yesterday: T,
    pub last_month: T,
    pub last_week: T,
    pub yearly_high: T,
    pub yearly_low: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub error_code: u64,
}

impl FearGreedData {
    pub fn get_latest_item(&self) -> Option<&FearGreedItem> {
        self.data_list.last()
    }

    pub fn get_items_in_date_range(&self, start: u64, end: u64) -> Vec<&FearGreedItem> {
        self.data_list.iter().filter(|x| x.timestamp >= start && x.timestamp <= end).collect()
    }
}

// Alt Season Index Models
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AltSeasonData {
    pub points: Vec<AltSeasonPoint>,
    pub historical_values: HistoricalValues<AltSeasonPoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AltSeasonPoint {
    pub name: String,
    #[serde(rename = "altcoinIndex")]
    pub altcoin_index: String,
    #[serde(rename = "altcoinMarketcap")]
    pub altcoin_marketcap: String,
    pub timestamp: String,
}

impl AltSeasonData {
    pub fn get_latest_point(&self) -> Option<&AltSeasonPoint> {
        self.points.last()
    }

    pub fn get_points_in_date_range(&self, start: u64, end: u64) -> Vec<&AltSeasonPoint> {
        self.points
            .iter()
            .filter(|x| {
                let timestamp = x.timestamp.parse::<u64>().unwrap_or(0);
                timestamp >= start && timestamp <= end
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_fear_greed_response() {
        let json_data = include_str!("test/fear_greed.json");
        let response: ApiResponse<FearGreedData> = serde_json::from_str(json_data).unwrap();

        // Check that data list exists and get the last item
        assert!(response.data.data_list.len() > 2);
        let last_item_index = response.data.data_list.len() - 1;

        assert_eq!(response.data.data_list[last_item_index].score, 27);
        assert_eq!(response.data.data_list[last_item_index].name, "Fear");
        assert_eq!(response.data.data_list[last_item_index].timestamp, 1743851290);

        assert_eq!(response.data.historical_values.now.score, 27);
        assert_eq!(response.data.historical_values.yesterday.score, 25);

        assert_eq!(response.status.error_code, 0);
    }

    #[test]
    fn test_decode_alt_season_response() {
        let json_data = include_str!("test/alt_index.json");
        let response: ApiResponse<AltSeasonData> = serde_json::from_str(json_data).unwrap();

        // Check that points exist and get the last item
        assert!(response.data.points.len() > 2);
        let last_point_index = response.data.points.len() - 1;

        assert_eq!(response.data.points[last_point_index].name, "Bitcoin Season");
        assert_eq!(response.data.points[last_point_index].altcoin_index, "15");
        assert_eq!(response.data.points[last_point_index].timestamp, "1743857664");

        assert_eq!(response.data.historical_values.now.altcoin_index, "15");
        assert_eq!(response.data.historical_values.yesterday.altcoin_index, "16");
        assert_eq!(response.data.historical_values.yearly_high.name, "Altcoin Season");

        assert_eq!(response.status.error_code, 0);
    }
}
