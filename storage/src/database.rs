use diesel::dsl::sql;
use diesel::{Connection, upsert::excluded};
use diesel::pg::PgConnection;
use crate::models::*;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");
use primitives::asset_price::ChartPeriod;
pub struct DatabaseClient {
    connection: PgConnection,
}

impl DatabaseClient {
    pub fn new(database_url: &str) -> Self {
        let connection = PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self{connection}
    }

    pub fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error> {
        use crate::schema::nodes::dsl::*;
        nodes
            .select(Node::as_select())
            .load(&mut self.connection)
    }

    pub fn get_nodes_version(&mut self) -> Result<i32, diesel::result::Error> {
        let version = self.get_nodes()?
            .iter()
            .map(|x| x.id)
            .collect::<Vec<i32>>()
            .iter()
            .sum();
        Ok(version)
    }

    pub fn get_versions(&mut self) -> Result<Vec<Version>, diesel::result::Error> {
        use crate::schema::versions::dsl::*;
        versions
            .order(id.asc())
            .select(Version::as_select())
            .load(&mut self.connection)
    }

    pub fn get_tokenlists(&mut self) -> Result<Vec<TokenList>, diesel::result::Error> {
        use crate::schema::tokenlists::dsl::*;
        tokenlists
            .select(TokenList::as_select())
            .load(&mut self.connection)
    }

    pub fn set_tokenlist_version(&mut self, _chain: String, _version: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::tokenlists::dsl::*;
        diesel::update(tokenlists)
            .filter(chain.eq(_chain))
            .set(version.eq(_version))
            .execute(&mut self.connection)
    }

    pub fn get_fiat_assets(&mut self) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets
            .select(FiatAsset::as_select())
            .load(&mut self.connection)
    }
    pub fn get_fiat_assets_for_asset_id(&mut self, asset_id: &str) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets
            .filter(asset.eq(asset_id))
            .select(FiatAsset::as_select())
            .load(&mut self.connection)
    }

    pub fn get_coin_id(&mut self, asset_id_value: &str) ->  Result<String, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        let result = prices
            .filter(asset_id.eq(asset_id_value))
            .select(Price::as_select())
            .load(&mut self.connection);
        return Ok(result.unwrap().first().unwrap().clone().coin_id)
    }

    pub fn set_prices(&mut self, asset_prices: Vec<Price>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices)
            .values(&asset_prices)
            .on_conflict(asset_id)
            .do_update()
            .set((
                price.eq(excluded(price)),
                coin_id.eq(excluded(coin_id)),
                price_change_percentage_24h.eq(excluded(price_change_percentage_24h)),
                market_cap.eq(excluded(market_cap)),
                market_cap_rank.eq(excluded(market_cap_rank)),
                total_volume.eq(excluded(total_volume)),
            ))
            .execute(&mut self.connection)
    }

    pub fn get_prices(&mut self) ->  Result<Vec<Price>, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices
            .select(Price::as_select())
            .load(&mut self.connection)
    }

    pub fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        diesel::insert_into(fiat_rates)
            .values(&rates)
            .on_conflict(symbol)
            .do_update()
            .set((
                rate.eq(excluded(rate)),
            ))
            .execute(&mut self.connection)
    }

    pub fn get_fiat_rates(&mut self) ->  Result<Vec<FiatRate>, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates
            .select(FiatRate::as_select())
            .load(&mut self.connection)
    }

    pub fn set_charts(&mut self, values: Vec<Chart>) -> Result<usize, diesel::result::Error> {
        use crate::schema::charts::dsl::*;
        diesel::insert_into(charts)
            .values(&values)
            .on_conflict((coin_id, date))
            .do_update()
            .set((
                price.eq(excluded(price)),
            ))
            .execute(&mut self.connection)
    }

    pub fn get_charts_prices(&mut self, coin_id_value: &str, period: &ChartPeriod) -> Result<Vec<ChartResult>, diesel::result::Error> {
        use crate::schema::charts::dsl::*;
        let date_selection = format!("date_trunc('{}', date)", self.period_sql(period.clone()));
        return charts
            .select(
                (sql::<diesel::sql_types::Timestamp>(date_selection.as_str()),
                (sql::<diesel::sql_types::Double>("AVG(price)")),
            ))
            .filter(coin_id.eq(coin_id_value))
            .filter(               
                 sql::<diesel::sql_types::Bool>(format!("date >= now() - INTERVAL '{} minutes'", self.period_minutes(period.clone())).as_str()),
            )
            .group_by(
                sql::<diesel::sql_types::Timestamp>(date_selection.as_str()),
            )
            .order(sql::<diesel::sql_types::Timestamp>("date_trunc").desc())
            .load(&mut self.connection);
    }

    fn period_sql(&self, period: ChartPeriod) -> &str {
        match period {
            ChartPeriod::Hour => "minute",
            ChartPeriod::Day => "minute",
            ChartPeriod::Week => "hour",
            ChartPeriod::Month => "hour",
            ChartPeriod::Quarter => "day",
            ChartPeriod::Year => "day",
            ChartPeriod::All => "day",
        }
    }

    fn period_minutes(&self, period: ChartPeriod) -> i32 {
        match period {
            ChartPeriod::Hour => 60,
            ChartPeriod::Day => 1440,
            ChartPeriod::Week => 10_080,
            ChartPeriod::Month => 43_200,
            ChartPeriod::Quarter => 131_400,
            ChartPeriod::Year => 525_600,
            ChartPeriod::All => 10_525_600,
        }
    }

    pub fn set_version(&mut self, version: Version) -> Result<usize, diesel::result::Error> {
        use crate::schema::versions::dsl::*;
        diesel::insert_into(versions)
            .values(&version)
            .on_conflict(platform)
            .do_update()
            .set((
                production.eq(excluded(production)),
                beta.eq(excluded(beta)),
                alpha.eq(excluded(alpha)),
            ))
            .execute(&mut self.connection)
    }

    pub fn migrations(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
}