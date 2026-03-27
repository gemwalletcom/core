use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use futures::{future::try_join_all, try_join};
use gem_client::Client;
use primitives::{
    ChartPeriod,
    chart::ChartCandleStick,
    perpetual::{PerpetualData, PerpetualPositionsSummary},
    portfolio::PerpetualPortfolio,
};

use crate::{
    config::HypercoreConfig,
    models::{perp_dex::PerpDex, position::AssetPositions},
    provider::perpetual_mapper::{
        map_account_summary_aggregate, map_candlesticks, map_perpetual_portfolio, map_perpetuals_data, map_positions, merge_perpetual_portfolios, merge_positions_summaries,
    },
    rpc::client::HyperCoreClient,
};

fn map_active_dex_entries(perp_dexs: &[Option<PerpDex>]) -> Vec<(u32, Option<String>)> {
    perp_dexs
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if index == 0 {
                return Some((0, None));
            }
            let dex = entry.as_ref()?;
            if !dex.is_available() {
                return None;
            }
            Some((index as u32, Some(dex.name.clone())))
        })
        .collect()
}

impl<C: Client> HyperCoreClient<C> {
    async fn get_active_dex_entries(&self) -> Result<Vec<(u32, Option<String>)>, Box<dyn Error + Sync + Send>> {
        Ok(map_active_dex_entries(&self.get_perp_dexs().await?))
    }

    async fn get_positions_for_dex(&self, address: &str, dex: &(u32, Option<String>)) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let (positions, orders) = try_join!(
            self.get_clearinghouse_state_by_dex(address, dex.1.as_deref()),
            self.get_open_orders_by_dex(address, dex.1.as_deref()),
        )?;
        Ok(map_positions(positions, address, &orders, dex.0))
    }

    async fn get_portfolio_for_dex(&self, address: &str, dex: &(u32, Option<String>)) -> Result<(PerpetualPortfolio, AssetPositions), Box<dyn Error + Sync + Send>> {
        let (response, positions) = try_join!(
            self.get_perpetual_portfolio_by_dex(address, dex.1.as_deref()),
            self.get_clearinghouse_state_by_dex(address, dex.1.as_deref()),
        )?;
        Ok((map_perpetual_portfolio(response, &positions), positions))
    }
}

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let dex_entries = self.get_active_dex_entries().await?;
        let requests = dex_entries.iter().map(|dex| self.get_positions_for_dex(&address, dex));
        let summaries = try_join_all(requests).await?;
        Ok(merge_positions_summaries(summaries))
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let dex_entries = self.get_active_dex_entries().await?;
        let requests = dex_entries.iter().map(|dex| self.get_metadata_by_dex(dex.1.as_deref()));
        let metadata = try_join_all(requests).await?;

        Ok(dex_entries.iter().zip(metadata).flat_map(|(dex, meta)| map_perpetuals_data(meta, dex.0)).collect())
    }

    async fn get_perpetual_candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        let interval = match period {
            ChartPeriod::Hour => "1m",
            ChartPeriod::Day => "30m",
            ChartPeriod::Week => "4h",
            ChartPeriod::Month => "12h",
            ChartPeriod::Year => "1w",
            ChartPeriod::All => "1M",
        };

        let end_time = chrono::Utc::now().timestamp() * 1000;
        let start_time = match period {
            ChartPeriod::Hour => end_time - 60 * 60 * 1000,
            ChartPeriod::Day => end_time - 24 * 60 * 60 * 1000,
            ChartPeriod::Week => end_time - 7 * 24 * 60 * 60 * 1000,
            ChartPeriod::Month => end_time - 30 * 24 * 60 * 60 * 1000,
            ChartPeriod::Year => end_time - 365 * 24 * 60 * 60 * 1000,
            ChartPeriod::All => 0,
        };

        let candlesticks = self.get_candlesticks(&symbol, interval, start_time, end_time).await?;
        Ok(map_candlesticks(candlesticks))
    }

    async fn get_perpetual_portfolio(&self, address: String) -> Result<PerpetualPortfolio, Box<dyn Error + Sync + Send>> {
        let dex_entries = self.get_active_dex_entries().await?;
        let requests = dex_entries.iter().map(|dex| self.get_portfolio_for_dex(&address, dex));
        let results = try_join_all(requests).await?;
        let (portfolios, positions): (Vec<_>, Vec<_>) = results.into_iter().unzip();
        let account_summary = Some(map_account_summary_aggregate(&positions));
        Ok(merge_perpetual_portfolios(portfolios, account_summary))
    }

    async fn get_perpetual_referred_addresses(&self) -> Result<Vec<String>, Box<dyn Error + Sync + Send>> {
        let config = HypercoreConfig::default();
        let referral = self.get_referral(&config.builder_address).await?;
        let referral_states = referral.referrer_state.and_then(|s| s.data).and_then(|d| d.referral_states).unwrap_or_default();
        Ok(referral_states.into_iter().map(|r| r.user).collect())
    }
}

#[async_trait]
impl<C: Client> ChainAddressStatus for HyperCoreClient<C> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::PerpDex;

    #[test]
    fn test_map_active_dex_entries_filters_inactive() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "dex1".to_string(),
                ..PerpDex::mock()
            }),
            Some(PerpDex {
                name: "dex2".to_string(),
                is_active: Some(false),
                ..PerpDex::mock()
            }),
            Some(PerpDex {
                name: "dex3".to_string(),
                is_active: None,
                ..PerpDex::mock()
            }),
        ];

        let entries = map_active_dex_entries(&dexs);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], (0, None));
        assert_eq!(entries[1], (1, Some("dex1".to_string())));
        assert_eq!(entries[2], (3, Some("dex3".to_string())));
    }

    #[test]
    fn test_map_active_dex_entries_skips_empty_names() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "".to_string(),
                ..PerpDex::mock()
            }),
        ];

        let entries = map_active_dex_entries(&dexs);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], (0, None));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use std::collections::HashSet;

    use crate::provider::testkit::{TEST_ADDRESS, create_hypercore_test_client};
    use chain_traits::ChainPerpetual;
    use primitives::ChartPeriod;

    #[tokio::test]
    async fn test_hypercore_get_perp_dexs() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let dexs = client.get_perp_dexs().await?;

        assert!(!dexs.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_positions() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let summary = client.get_positions(TEST_ADDRESS.to_string()).await?;

        let ids: HashSet<_> = summary.positions.iter().map(|position| position.id.clone()).collect();
        assert_eq!(ids.len(), summary.positions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetuals_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let data = client.get_perpetuals_data().await?;

        assert!(!data.is_empty());

        let perpetual_ids: HashSet<_> = data.iter().map(|item| item.perpetual.id.clone()).collect();
        assert_eq!(perpetual_ids.len(), data.len());

        let asset_ids: HashSet<_> = data.iter().map(|item| item.asset.id.to_string()).collect();
        assert_eq!(asset_ids.len(), data.len());

        let identifiers: HashSet<_> = data.iter().map(|item| item.perpetual.identifier.clone()).collect();
        assert_eq!(identifiers.len(), data.len());

        let btc = data.iter().find(|item| item.perpetual.name == "BTC").unwrap();
        assert_eq!(btc.perpetual.identifier, "0");

        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_portfolio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, TEST_ADDRESS.to_string()).await?;

        assert!(portfolio.day.is_some() || portfolio.week.is_some() || portfolio.month.is_some() || portfolio.all_time.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_candlesticks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let candlesticks = client.get_perpetual_candlesticks("BTC".to_string(), ChartPeriod::Day).await?;

        assert!(!candlesticks.is_empty());
        Ok(())
    }
}
