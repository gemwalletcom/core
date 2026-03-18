use crate::DatabaseClient;
use crate::models::{NewPerpetualRow, PerpetualRow};
use crate::schema::{perpetuals, perpetuals_assets};
use chrono::NaiveDateTime;
use diesel::{prelude::*, upsert::excluded};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerpetualFilter {
    UpdatedSince(NaiveDateTime),
}

pub(crate) trait PerpetualsStore {
    fn perpetuals_update(&mut self, values: Vec<NewPerpetualRow>) -> Result<usize, diesel::result::Error>;

    fn get_perpetuals_for_asset(&mut self, asset_id_value: &str) -> Result<Vec<PerpetualRow>, diesel::result::Error>;

    fn get_perpetuals_by_filter(&mut self, filters: Vec<PerpetualFilter>) -> Result<Vec<PerpetualRow>, diesel::result::Error>;
}

impl PerpetualsStore for DatabaseClient {
    fn perpetuals_update(&mut self, values: Vec<NewPerpetualRow>) -> Result<usize, diesel::result::Error> {
        if values.is_empty() {
            return Ok(0);
        }
        diesel::insert_into(perpetuals::table)
            .values(&values)
            .on_conflict(perpetuals::id)
            .do_update()
            .set((
                perpetuals::name.eq(excluded(perpetuals::name)),
                perpetuals::provider.eq(excluded(perpetuals::provider)),
                perpetuals::asset_id.eq(excluded(perpetuals::asset_id)),
                perpetuals::price.eq(excluded(perpetuals::price)),
                perpetuals::price_percent_change_24h.eq(excluded(perpetuals::price_percent_change_24h)),
                perpetuals::open_interest.eq(excluded(perpetuals::open_interest)),
                perpetuals::volume_24h.eq(excluded(perpetuals::volume_24h)),
                perpetuals::funding.eq(excluded(perpetuals::funding)),
                perpetuals::leverage.eq(excluded(perpetuals::leverage)),
            ))
            .execute(&mut self.connection)
    }

    fn get_perpetuals_for_asset(&mut self, asset_id_value: &str) -> Result<Vec<PerpetualRow>, diesel::result::Error> {
        perpetuals::table
            .inner_join(perpetuals_assets::table.on(perpetuals::id.eq(perpetuals_assets::perpetual_id)))
            .filter(perpetuals_assets::asset_id.eq(asset_id_value))
            .select(PerpetualRow::as_select())
            .load(&mut self.connection)
    }

    fn get_perpetuals_by_filter(&mut self, filters: Vec<PerpetualFilter>) -> Result<Vec<PerpetualRow>, diesel::result::Error> {
        let mut query = perpetuals::table.into_boxed();

        for filter in filters {
            match filter {
                PerpetualFilter::UpdatedSince(value) => {
                    query = query.filter(perpetuals::updated_at.gt(value));
                }
            }
        }

        query.select(PerpetualRow::as_select()).load(&mut self.connection)
    }
}
