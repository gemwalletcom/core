use std::error::Error;

use super::{JitoTipFloor, JitoTipFloorEntry, JITO_TIP_FLOOR_URL};

#[derive(Default)]
pub struct JitoClient {
    client: reqwest::Client,
}

impl JitoClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn fetch_tip_floor(&self) -> Result<JitoTipFloor, Box<dyn Error + Send + Sync>> {
        let response = self.client.get(JITO_TIP_FLOOR_URL).send().await?;
        let entries: Vec<JitoTipFloorEntry> = response.json().await?;
        let entry = entries.first().ok_or("No tip floor data returned from Jito API")?;
        Ok(JitoTipFloor::from_entry(entry))
    }
}
