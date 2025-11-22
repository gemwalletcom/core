use async_trait::async_trait;
use std::error::Error;
use storage::Database;
use storage::models::Price;
use streamer::{PricesPayload, consumer::MessageConsumer};

pub struct StorePricesConsumer {
    pub database: Database,
}

impl StorePricesConsumer {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl MessageConsumer<PricesPayload, usize> for StorePricesConsumer {
    async fn should_process(&self, _payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: PricesPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        println!("StorePricesConsumer received {} prices", payload.prices.len());
        let prices: Vec<Price> = payload.prices.iter().map(|p| Price::from_price_data(p.clone())).collect();
        self.database.client()?.prices().set_prices(prices)?;
        Ok(payload.prices.len())
    }
}
