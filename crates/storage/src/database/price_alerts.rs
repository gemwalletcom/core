use chrono::NaiveDateTime;

use crate::{models::*, DatabaseClient};
use diesel::prelude::*;

impl DatabaseClient {
    pub fn get_price_alerts_with_prices(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<(PriceAlert, Price)>, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        use crate::schema::prices;
        use crate::schema::prices_assets;

        price_alerts
            .filter(last_notified_at.lt(after_notified_at).or(last_notified_at.is_null()))
            .inner_join(prices_assets::table.on(asset_id.eq(prices_assets::asset_id)))
            .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
            .select((PriceAlert::as_select(), Price::as_select()))
            .distinct()
            .load(&mut self.connection)
    }

    pub fn get_price_alerts_for_device_id(&mut self, _device_id: i32) -> Result<Vec<PriceAlert>, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        price_alerts
            .filter(device_id.eq(_device_id))
            .select(PriceAlert::as_select())
            .load(&mut self.connection)
    }

    pub fn add_price_alerts(&mut self, values: Vec<NewPriceAlert>) -> Result<usize, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        diesel::insert_into(price_alerts)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn delete_price_alerts(&mut self, _device_id: i32, ids: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        diesel::delete(price_alerts.filter(device_id.eq(_device_id).and(identifier.eq_any(ids)))).execute(&mut self.connection)
    }

    pub fn update_price_alerts_set_notified_at(&mut self, ids: Vec<String>, _last_notified_at: NaiveDateTime) -> Result<usize, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        diesel::update(price_alerts)
            .filter(identifier.eq_any(&ids))
            .set(last_notified_at.eq(_last_notified_at))
            .execute(&mut self.connection)
    }
}
