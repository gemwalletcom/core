use crate::models::asset::AssetLink;
use crate::models::*;
use crate::schema::{devices, fiat_providers, prices_assets, transactions_addresses};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::associations::HasTable;
use diesel::dsl::count;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{upsert::excluded, Connection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use price_alert::NewPriceAlert;
use primitives::chain::Chain;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/migrations");
use self::price::PriceAsset;
use primitives::{AssetType, TransactionsFetchOption};

pub struct DatabaseClient {
    connection: PgConnection,
}

impl DatabaseClient {
    pub fn new(database_url: &str) -> Self {
        let connection = PgConnection::establish(database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        Self { connection }
    }

    pub fn get_nodes(&mut self) -> Result<Vec<Node>, diesel::result::Error> {
        use crate::schema::nodes::dsl::*;
        nodes.select(Node::as_select()).load(&mut self.connection)
    }

    pub fn get_releases(&mut self) -> Result<Vec<Release>, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        releases.order(id.asc()).select(Release::as_select()).load(&mut self.connection)
    }

    pub fn get_tokenlists(&mut self) -> Result<Vec<TokenList>, diesel::result::Error> {
        use crate::schema::tokenlists::dsl::*;
        tokenlists.select(TokenList::as_select()).load(&mut self.connection)
    }

    pub fn set_tokenlist_version(&mut self, _chain: String, _version: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::tokenlists::dsl::*;
        diesel::update(tokenlists)
            .filter(chain.eq(_chain))
            .set(version.eq(_version))
            .execute(&mut self.connection)
    }

    // fiat
    pub fn add_fiat_assets(&mut self, values: Vec<FiatAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        diesel::insert_into(fiat_assets)
            .values(values)
            .on_conflict((id,))
            .do_update()
            .set((
                asset_id.eq(excluded(asset_id)),
                symbol.eq(excluded(symbol)),
                network.eq(excluded(network)),
                token_id.eq(excluded(token_id)),
                enabled_by_provider.eq(excluded(enabled_by_provider)),
            ))
            .execute(&mut self.connection)
    }

    pub fn add_fiat_providers(&mut self, values: Vec<FiatProvider>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_providers::dsl::*;
        diesel::insert_into(fiat_providers)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_fiat_transaction(&mut self, transaction: FiatTransaction) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_transactions::dsl::*;

        let update = FiatTransactionUpdate {
            status: transaction.status.clone(),
            transaction_hash: transaction.transaction_hash.clone(),
            address: transaction.address.clone(),
            fee_provider: transaction.fee_provider,
            fee_partner: transaction.fee_partner,
            fee_network: transaction.fee_network,
        };

        diesel::insert_into(fiat_transactions)
            .values(&transaction)
            .on_conflict((provider_id, provider_transaction_id))
            .do_update()
            .set(update)
            .execute(&mut self.connection)
    }

    pub fn get_fiat_assets(&mut self) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets.select(FiatAsset::as_select()).load(&mut self.connection)
    }

    pub fn get_fiat_assets_is_buyable(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_buyable.eq(true)).select(id).load(&mut self.connection)
    }

    pub fn get_fiat_assets_is_sellable(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_sellable.eq(true)).select(id).load(&mut self.connection)
    }

    pub fn get_fiat_assets_for_asset_id(&mut self, _asset_id: &str) -> Result<Vec<FiatAsset>, diesel::result::Error> {
        use crate::schema::fiat_assets::dsl::*;
        fiat_assets::table()
            .inner_join(fiat_providers::table)
            .filter(fiat_providers::enabled.eq(true))
            .filter(asset_id.eq(_asset_id))
            .select(FiatAsset::as_select())
            .load(&mut self.connection)
    }

    pub fn set_prices(&mut self, values: Vec<Price>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((
                price.eq(excluded(price)),
                price_change_percentage_24h.eq(excluded(price_change_percentage_24h)),
                all_time_high.eq(excluded(all_time_high)),
                all_time_high_date.eq(excluded(all_time_high_date)),
                all_time_low.eq(excluded(all_time_low)),
                all_time_low_date.eq(excluded(all_time_low_date)),
                market_cap.eq(excluded(market_cap)),
                market_cap_rank.eq(excluded(market_cap_rank)),
                total_volume.eq(excluded(total_volume)),
                circulating_supply.eq(excluded(circulating_supply)),
                total_supply.eq(excluded(total_supply)),
                max_supply.eq(excluded(max_supply)),
                last_updated_at.eq(excluded(last_updated_at)),
            ))
            .execute(&mut self.connection)
    }

    pub fn set_prices_simple(&mut self, values: Vec<Price>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::insert_into(prices)
            .values(&values)
            .on_conflict(id)
            .do_update()
            .set((
                price.eq(excluded(price)),
                price_change_percentage_24h.eq(excluded(price_change_percentage_24h)),
                market_cap.eq(excluded(market_cap)),
                total_volume.eq(excluded(total_volume)),
                last_updated_at.eq(excluded(last_updated_at)),
            ))
            .execute(&mut self.connection)
    }

    pub fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        diesel::insert_into(prices_assets)
            .values(&values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn get_prices(&mut self) -> Result<Vec<Price>, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices.order(market_cap.desc()).select(Price::as_select()).load(&mut self.connection)
    }

    pub fn get_prices_assets(&mut self) -> Result<Vec<PriceAsset>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.select(PriceAsset::as_select()).load(&mut self.connection)
    }

    pub fn get_price(&mut self, asset_id: &str) -> Result<Price, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        prices
            .inner_join(prices_assets::table)
            .filter(prices_assets::asset_id.eq(asset_id))
            .select(Price::as_select())
            .first(&mut self.connection)
    }

    pub fn get_prices_id_for_asset_id(&mut self, id: &str) -> Result<Vec<PriceAsset>, diesel::result::Error> {
        use crate::schema::prices_assets::dsl::*;
        prices_assets.filter(asset_id.eq(id)).select(PriceAsset::as_select()).load(&mut self.connection)
    }

    pub fn delete_prices_updated_at_before(&mut self, time: NaiveDateTime) -> Result<usize, diesel::result::Error> {
        use crate::schema::prices::dsl::*;
        diesel::delete(prices.filter(last_updated_at.lt(time).or(last_updated_at.is_null()))).execute(&mut self.connection)
    }

    pub fn set_fiat_rates(&mut self, rates: Vec<FiatRate>) -> Result<usize, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        diesel::insert_into(fiat_rates)
            .values(&rates)
            .on_conflict(symbol)
            .do_update()
            .set(rate.eq(excluded(rate)))
            .execute(&mut self.connection)
    }

    pub fn get_fiat_rates(&mut self) -> Result<Vec<FiatRate>, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.select(FiatRate::as_select()).load(&mut self.connection)
    }

    pub fn get_fiat_rate(&mut self, currency: &str) -> Result<FiatRate, diesel::result::Error> {
        use crate::schema::fiat_rates::dsl::*;
        fiat_rates.filter(symbol.eq(currency)).select(FiatRate::as_select()).first(&mut self.connection)
    }

    pub fn add_releases(&mut self, values: Vec<Release>) -> Result<usize, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        diesel::insert_into(releases)
            .values(&values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_release(&mut self, release: Release) -> Result<usize, diesel::result::Error> {
        use crate::schema::releases::dsl::*;
        diesel::insert_into(releases)
            .values(&release)
            .on_conflict(platform_store)
            .do_update()
            .set(version.eq(excluded(version)))
            .execute(&mut self.connection)
    }

    pub fn add_device(&mut self, device: UpdateDevice) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::insert_into(devices)
            .values(&device)
            .returning(Device::as_returning())
            .get_result(&mut self.connection)
    }

    pub fn get_device_by_id(&mut self, _id: i32) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(id.eq(_id)).select(Device::as_select()).first(&mut self.connection)
    }

    pub fn get_device(&mut self, _device_id: &str) -> Result<Device, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(device_id.eq(_device_id)).select(Device::as_select()).first(&mut self.connection)
    }

    pub fn get_device_token(&mut self, _device_id: &str) -> Result<String, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        devices.filter(device_id.eq(_device_id)).select(token).first(&mut self.connection)
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
        diesel::delete(devices.filter(device_id.eq(_device_id))).execute(&mut self.connection)
    }

    pub fn update_device_is_push_enabled(&mut self, _device_id: &str, value: bool) -> Result<usize, diesel::result::Error> {
        use crate::schema::devices::dsl::*;
        diesel::update(devices)
            .filter(device_id.eq(_device_id))
            .set(is_push_enabled.eq(value))
            .execute(&mut self.connection)
    }

    // Delete subscriptions for inactive devices
    pub fn delete_devices_subscriptions_after_days(&mut self, days: i64) -> Result<usize, diesel::result::Error> {
        let cutoff_date = Utc::now() - Duration::days(days);

        let device_ids_query = crate::schema::devices::table
            .filter(crate::schema::devices::updated_at.lt(cutoff_date.naive_utc()))
            .select(crate::schema::devices::id);

        diesel::delete(crate::schema::subscriptions::table.filter(crate::schema::subscriptions::device_id.eq_any(device_ids_query)))
            .execute(&mut self.connection)
    }

    pub fn get_parser_state(&mut self, _chain: Chain) -> Result<ParserState, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state
            .filter(chain.eq(_chain.as_ref()))
            .select(ParserState::as_select())
            .first(&mut self.connection)
    }

    pub fn add_parser_state(&mut self, _chain: Chain) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::insert_into(parser_state)
            .values(chain.eq(_chain.as_ref()))
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn get_parser_states(&mut self) -> Result<Vec<ParserState>, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        parser_state.select(ParserState::as_select()).load(&mut self.connection)
    }

    pub fn set_parser_state_latest_block(&mut self, _chain: Chain, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(_chain.as_ref()))
            .set(latest_block.eq(block))
            .execute(&mut self.connection)
    }

    pub fn set_parser_state_current_block(&mut self, _chain: Chain, block: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::parser_state::dsl::*;
        diesel::update(parser_state.find(_chain.as_ref()))
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
                .filter(address.eq(subscription.address)),
        )
        .execute(&mut self.connection)
    }

    // distinct_on is used to only select once subscription per user device
    pub fn get_subscriptions(&mut self, _chain: Chain, addresses: Vec<String>) -> Result<Vec<Subscription>, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;

        // exlcude addresses from subscriptions
        let exclude_addresses = self.get_subscriptions_exclude_addresses(addresses.clone())?;

        subscriptions
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq_any(addresses))
            .filter(address.ne_all(exclude_addresses))
            .distinct_on((device_id, chain, address))
            .select(Subscription::as_select())
            .load(&mut self.connection)
    }

    pub fn get_subscriptions_exclude_addresses(&mut self, addresses: Vec<String>) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        subscriptions_addresses_exclude
            .filter(address.eq_any(addresses))
            .select(address)
            .load(&mut self.connection)
    }

    pub fn add_subscriptions(&mut self, _subscriptions: Vec<Subscription>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions::dsl::*;
        diesel::insert_into(subscriptions)
            .values(&_subscriptions)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_transactions(
        &mut self,
        transactions_values: Vec<Transaction>,
        addresses_values: Vec<TransactionAddresses>,
    ) -> Result<bool, diesel::result::Error> {
        self.connection
            .build_transaction()
            .read_write()
            .run::<_, diesel::result::Error, _>(|conn: &mut PgConnection| {
                use crate::schema::transactions::dsl::*;
                let query1 = diesel::insert_into(transactions::table())
                    .values(transactions_values)
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
                    .execute(conn);

                if let Some(error) = query1.err() {
                    return Err(error);
                }

                use crate::schema::transactions_addresses::dsl::*;
                let query2 = diesel::insert_into(transactions_addresses::table())
                    .values(&addresses_values)
                    .on_conflict((
                        super::schema::transactions_addresses::transaction_id,
                        super::schema::transactions_addresses::address,
                        super::schema::transactions_addresses::asset_id,
                    ))
                    .do_nothing()
                    .execute(conn);

                if let Some(error) = query2.err() {
                    return Err(error);
                }

                Ok(true)
            })
    }

    pub fn get_transactions_by_device_id(
        &mut self,
        _device_id: &str,
        addresses: Vec<String>,
        chains: Vec<String>,
        options: TransactionsFetchOption,
    ) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;

        let mut query = transactions
            .into_boxed()
            .inner_join(transactions_addresses::table)
            .filter(transactions_addresses::chain_id.eq_any(chains.clone()))
            .filter(transactions_addresses::address.eq_any(addresses));

        if let Some(_asset_id) = options.asset_id {
            query = query.filter(asset_id.eq(_asset_id));
        }

        if let Some(from_timestamp) = options.from_timestamp {
            let datetime = DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc();
            query = query.filter(created_at.gt(datetime));
        }

        query.order(created_at.desc()).select(Transaction::as_select()).load(&mut self.connection)
    }

    pub fn get_transactions_by_hash(&mut self, _hash: &str) -> Result<Vec<Transaction>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        transactions
            .filter(hash.eq(_hash))
            .order(created_at.asc())
            .select(Transaction::as_select())
            .load(&mut self.connection)
    }

    pub fn get_transactions_addresses(&mut self, min_count: i64, limit: i64) -> Result<Vec<AddressChainIdResult>, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        transactions_addresses
            .select((address, chain_id))
            .group_by((address, chain_id))
            .having(count(address).gt(min_count))
            .order_by(count(address).desc())
            .limit(limit)
            .load::<AddressChainIdResult>(&mut self.connection)
    }

    pub fn add_subscriptions_address_exclude(&mut self, values: Vec<SubscriptionAddressExclude>) -> Result<usize, diesel::result::Error> {
        use crate::schema::subscriptions_addresses_exclude::dsl::*;
        diesel::insert_into(subscriptions_addresses_exclude)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn delete_transactions_addresses(&mut self, addresses: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        diesel::delete(transactions_addresses)
            .filter(address.eq_any(addresses))
            .execute(&mut self.connection)
    }

    pub fn get_transactions_without_addresses(&mut self, limit: i64) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        use crate::schema::transactions_addresses::dsl as addr;

        transactions
            .left_outer_join(addr::transactions_addresses.on(id.eq(addr::transaction_id)))
            .filter(addr::transaction_id.is_null())
            .select(id)
            .limit(limit)
            .load(&mut self.connection)
    }

    pub fn delete_transactions_by_ids(&mut self, ids: Vec<String>) -> Result<usize, diesel::result::Error> {
        use crate::schema::transactions::dsl::*;
        diesel::delete(transactions.filter(id.eq_any(ids))).execute(&mut self.connection)
    }

    pub fn get_asset(&mut self, asset_id: &str) -> Result<Asset, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(id.eq(asset_id)).select(Asset::as_select()).first(&mut self.connection)
    }

    pub fn get_asset_links(&mut self, _asset_id: &str) -> Result<Vec<AssetLink>, diesel::result::Error> {
        use crate::schema::assets_links::dsl::*;
        assets_links
            .filter(asset_id.eq(_asset_id))
            .select(AssetLink::as_select())
            .load(&mut self.connection)
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets
            .filter(id.eq_any(asset_ids))
            .filter(is_enabled.eq(true))
            .select(Asset::as_select())
            .load(&mut self.connection)
    }

    pub fn get_assets_list(&mut self) -> Result<Vec<Asset>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_enabled.eq(true)).select(Asset::as_select()).load(&mut self.connection)
    }

    pub fn get_assets_ids_by_device_id(
        &mut self,
        addresses: Vec<String>,
        chains: Vec<String>,
        from_timestamp: Option<u32>,
    ) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::transactions_addresses::dsl::*;
        let datetime = if let Some(from_timestamp) = from_timestamp {
            DateTime::from_timestamp(from_timestamp.into(), 0).unwrap().naive_utc()
        } else {
            DateTime::from_timestamp(0, 0).unwrap().naive_utc()
        };

        transactions_addresses
            .filter(address.eq_any(addresses))
            .filter(chain_id.eq_any(chains))
            .filter(created_at.gt(datetime))
            .order((asset_id, created_at.desc()))
            .distinct_on(asset_id)
            .select(asset_id)
            .load(&mut self.connection)
    }

    pub fn add_assets(&mut self, values: Vec<Asset>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::insert_into(assets)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn update_asset_rank(&mut self, asset_id: &str, _rank: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::update(assets)
            .filter(id.eq(asset_id))
            .filter(rank.eq(0))
            .set(rank.eq(_rank))
            .execute(&mut self.connection)
    }

    pub fn add_assets_types(&mut self, values: Vec<AssetType>) -> Result<usize, diesel::result::Error> {
        let values = values
            .iter()
            .map(|x| super::models::AssetType { id: x.as_ref().to_owned() })
            .collect::<Vec<_>>();

        use crate::schema::assets_types::dsl::*;
        diesel::insert_into(assets_types)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    pub fn add_assets_links(&mut self, values: Vec<AssetLink>) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets_links::dsl::*;
        diesel::insert_into(assets_links)
            .values(values)
            .on_conflict((asset_id, name))
            .do_update()
            .set((url.eq(excluded(url)),))
            .execute(&mut self.connection)
    }

    pub fn get_scan_address(&mut self, _chain: Chain, value: &str) -> Result<ScanAddress, diesel::result::Error> {
        use crate::schema::scan_addresses::dsl::*;
        scan_addresses
            .filter(chain.eq(_chain.as_ref()))
            .filter(address.eq(value))
            .select(ScanAddress::as_select())
            .first(&mut self.connection)
    }

    // swap

    pub fn get_swap_assets(&mut self) -> Result<Vec<String>, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        assets.filter(is_swappable.eq(true)).select(id).load(&mut self.connection)
    }

    pub fn get_swap_assets_version(&mut self) -> Result<i32, diesel::result::Error> {
        let assets = self.get_swap_assets()?;
        Ok(assets.len() as i32)
    }

    pub fn set_swap_enabled(&mut self, asset_ids: Vec<String>, value: bool) -> Result<usize, diesel::result::Error> {
        use crate::schema::assets::dsl::*;
        diesel::update(assets)
            .filter(id.eq_any(&asset_ids))
            .set(is_swappable.eq(value))
            .execute(&mut self.connection)
    }

    pub fn add_chains(&mut self, _chains: Vec<String>) -> Result<usize, diesel::result::Error> {
        let values = _chains.iter().map(|chain| super::models::Chain { id: chain.clone() }).collect::<Vec<_>>();

        use crate::schema::chains::dsl::*;
        diesel::insert_into(chains)
            .values(values)
            .on_conflict_do_nothing()
            .execute(&mut self.connection)
    }

    // price alerts
    pub fn get_price_alerts(&mut self, after_notified_at: NaiveDateTime) -> Result<Vec<PriceAlert>, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        price_alerts
            .filter(last_notified_at.lt(after_notified_at).or(last_notified_at.is_null()))
            .select(PriceAlert::as_select())
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

    pub fn delete_price_alerts(&mut self, _device_id: i32, asset_ids: Vec<&str>) -> Result<usize, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        diesel::delete(price_alerts.filter(device_id.eq(_device_id).and(asset_id.eq_any(asset_ids)))).execute(&mut self.connection)
    }

    pub fn update_price_alerts_set_notified_at(&mut self, ids: Vec<i32>, _last_notified_at: NaiveDateTime) -> Result<usize, diesel::result::Error> {
        use crate::schema::price_alerts::dsl::*;
        diesel::update(price_alerts)
            .filter(id.eq_any(&ids))
            .set(last_notified_at.eq(_last_notified_at))
            .execute(&mut self.connection)
    }

    pub fn migrations(&mut self) {
        self.connection.run_pending_migrations(MIGRATIONS).unwrap();
    }
}
