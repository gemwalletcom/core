use chrono::{DateTime, NaiveDate, TimeZone, Utc};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInformation {
    pub title: String,
    pub description: String,
    pub summary: String,
    pub score: Option<f64>,
    pub ratings: Option<f64>,
    pub updated: i64,
    pub app_id: String,
    pub version: String,
    pub released: String,
}

impl AppInformation {
    pub fn release_date(&self) -> DateTime<Utc> {
        let parsed_date = NaiveDate::parse_from_str(self.released.as_str(), "%b %d, %Y").expect("Failed to parse date");
        let naive_datetime = parsed_date.and_hms_opt(0, 0, 0).unwrap_or_default();
        let datetime_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
        datetime_utc
    }

    pub fn updated_date(&self) -> DateTime<Utc> {
        let datetime_utc = Utc.timestamp_millis_opt(self.updated);
        datetime_utc.unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSearch {
    pub app_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: String,
    pub user_name: String,
    pub score: Option<i64>,
    pub text: String,
    pub version: Option<String>,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Results<T> {
    pub results: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data<T> {
    pub data: T,
}
