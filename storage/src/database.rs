use chrono::{Utc, Duration, NaiveDateTime};
use diesel::dsl::sql;
use diesel::{Connection, upsert::excluded};
use diesel::pg::PgConnection;
use primitives::chain::Chain;
use crate::models::*;
use crate::models::asset::AssetDetail;
use crate::schema::devices;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../storage/src/migrations");
use primitives::{TransactionsFetchOption, ChartPeriod};

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

    pub fn get_fiat_assets_version(&mut self) -> Result<i32, diesel::result::Error> {
        let version = self.get_fiat_assets()?
            .iter()
            .map(|x| x.id)
            .collect::<Vec<i32>>()
            .iter()
            .sum();
        Ok(version)
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
        Ok(result.unwrap().first().unwrap().clone().coin_id)
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
                circulating_supply.eq(excluded(circulating_supply)),
                total_supply.eq(excluded(total_supply)),
                max_supply.eq(excluded(max_supply)),
            ))
            .execute(&mut self.connection)
    }

    pub fn get_prices(&mut self) ->  Result<Vec<Price>, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices
            .select(Price::as_select())
            .load(&mut self.connection)
    }

    pub fn get_price(&mut self, _asset_id: &str) ->  Result<Price, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices
            .filter(asset_id.eq(_asset_id))
            .select(Price::as_select())
            .first(&mut self.connection)
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
        let date_selection = format!("date_bin('{}', date, timestamp '2000-01-01')", self.period_sql(period.clone()));
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
                sql::<diesel::sql_types::Numeric>("1"),
            )
            .order(sql::<diesel::sql_types::Numeric>("1").desc())
            .load(&mut self.connection);
    }

    fn period_sql(&self, period: ChartPeriod) -> &str {
        match period {
            ChartPeriod::Hour => "1 minutes",
            ChartPeriod::Day => "15 minutes",
            ChartPeriod::Week => "1 hour",
            ChartPeriod::Month => "6 hour",
            ChartPeriod::Quarter => "1 day",
            ChartPeriod::Year => "3 day",
            ChartPeriod::All => "3 day",
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

    pub fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
            diesel::insert_into(devices)
            .values(device)
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    pub fn get_device_by_id(&mut self, _id: i32) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices
            .filter(id.eq(_id))
            .select(Device::as_select())
            .first(&mut self.connection)
    }

    pub fn get_device(&mut self, _device_id: &str) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices
            .filter(device_id.eq(_device_id))
            .select(Device::as_select())
            .first(&mut self.connection)
    }

    pub fn get_device_token(&mut self, _device_id: &str) -> Result<String, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices
            .filter(device_id.eq(_device_id))
            .select(token)
            .first(&mut self.connection)
    }

    pub fn update_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(device.clone().device_id))
            .set(device)
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    pub fn delete_device(&mut self, _device_id: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::delete(
            devices
            .filter(device_id.eq(_device_id))
        ).execute(&mut self.connection)
    }

    pub fn update_device_is_push_enabled(&mut self, _device_id: &str, value: bool) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(_device_id))
            .set(
                is_push_enabled.eq(value)
            )
            .execute(&mut self.connection)
    }

    pub fn delete_devices_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        let cutoff_date = Utc::now() - Duration::days(days);
        diesel::delete(devices.filter(updated_at.lt(cutoff_date.naive_utc())))
            .execute(&mut self.connection)
    }

    pub fn get_parser_state(&mut self, _chain: Chain) -> Result<ParserState, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state
            .filter(chain.eq(_chain.as_str()))
            .select(ParserState::as_select())
            .first(&mut self.connection)
    }

    pub fn add_parser_state(&mut self, _chain: Chain) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::insert_into(parser_state)
            .values(chain.eq(_chain.as_str()))
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn get_parser_states(&mut self) -> Result<Vec<ParserState>, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state
            .select(ParserState::as_select())
            .load(&mut self.connection)
    }

    pub fn set_parser_state_latest_block(&mut self, _chain: Chain, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
            diesel::update(parser_state.find(_chain.as_str()))
                .set(latest_block.eq(block))
                .execute(&mut self.connection)
    }

    pub fn set_parser_state_current_block(&mut self, _chain: Chain, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
            diesel::update(parser_state.find(_chain.as_str()))
                .set(current_block.eq(block))
                .execute(&mut self.connection)
    }

    pub fn get_subscriptions_by_device_id(&mut self, _device_id: &str) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        subscriptions
            .inner_join(devices::table)
            .filter(devices::device_id.eq(_device_id))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn get_subscriptions_by_device_id_wallet_index(&mut self, _device_id: &str, _wallet_index: i32) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        subscriptions
            .filter(wallet_index.eq(_wallet_index))
            .inner_join(devices::table)
            .filter(devices::device_id.eq(_device_id))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn delete_subscription(&mut self, subscription: Subscription) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::delete(
            subscriptions
            .filter(device_id.eq(subscription.device_id))
            .filter(chain.eq(subscription.chain))
            .filter(address.eq(subscription.address))
        ).execute(&mut self.connection)
    }

    pub fn get_subscriptions(&mut self, _chain: Chain, addresses: Vec<String>) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        subscriptions
            .filter(chain.eq(_chain.as_str()))
            .filter(address.eq_any(addresses))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn add_subscriptions(&mut self, _subscriptions: Vec<Subscription>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::insert_into(subscriptions)
            .values(&_subscriptions)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_transactions(&mut self, _transactions: Vec<Transaction>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        diesel::insert_into(transactions)
            .values(&_transactions)
            .on_conflict((chain, hash))
            .do_update()
            .set((
                block_number.eq(excluded(block_number)),
                sequence.eq(excluded(sequence)),
                fee.eq(excluded(fee)),
                fee_asset_id.eq(excluded(fee_asset_id)),
                memo.eq(excluded(memo)),
                updated_at.eq(excluded(updated_at)),
            ))
            .execute(&mut self.connection)
    }
    
    pub fn get_transactions_by_device_id(&mut self, _device_id: &str, addresses: Vec<String>, chains: Vec<String>, options: TransactionsFetchOption) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        
        let mut query = crate::schema::transactions::table.into_boxed();
        query = query
            .filter(chain.eq_any(chains.clone()))
            .filter(from_address.eq_any(addresses.clone()))
            .or_filter(to_address.eq_any(addresses));
        
        if let Some(_asset_id) = options.asset_id  {
            query = query.filter(asset_id.eq(_asset_id));
        }

        if let Some(from_timestamp) = options.from_timestamp  {
            let datetime = NaiveDateTime::from_timestamp_opt(from_timestamp.into(), 0).unwrap();
            query = query.filter(created_at.gt(datetime));
        }

        query
            .order(created_at.desc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }

    pub fn get_transactions_by_hash(&mut self, _hash: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        transactions
            .filter(hash.eq(_hash))
            .order(created_at.asc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }

    pub fn get_asset(&mut self, asset_id: String) -> Result<Asset, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets
            .filter(id.eq(asset_id))
            .select(Asset::as_select())
            .first(&mut self.connection)
    }

    pub fn get_asset_details(&mut self, _asset_id: String) -> Result<AssetDetail, diesel::result::Error> {
        use crate::schema::assets_details::dsl::*;
        assets_details
            .filter(asset_id.eq(_asset_id))
            .select(AssetDetail::as_select())
            .first(&mut self.connection)
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets
            .filter(id.eq_any(asset_ids))
            .select(Asset::as_select())
            .load(&mut self.connection)
    }

    pub fn get_assets_search(&mut self, query: &str) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        let ilike_expression = format!("{}%", query);
        assets
            .order(rank.desc())
            .filter(name.ilike(ilike_expression.clone()))
            .or_filter(symbol.ilike(ilike_expression.clone()))
            .or_filter(token_id.ilike(ilike_expression.clone()))
            .select(Asset::as_select())
            .load(&mut self.connection)
    }

    pub fn add_assets(&mut self, _assets: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::insert_into(assets)
            .values(&_assets)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_assets_details(&mut self, _assets_details: Vec<AssetDetail>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets_details::dsl::*;
        diesel::insert_into(assets_details)
            .values(&_assets_details)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn get_scan_address(&mut self, _address: &str) ->  Result<ScanAddress, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        scan_addresses
            .filter(address.eq(_address))
            .select(ScanAddress::as_select())
            .first(&mut self.connection)
    }

    pub fn migrations(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
}