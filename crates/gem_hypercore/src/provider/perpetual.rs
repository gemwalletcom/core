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
    provider::{
        hip3_mapper::{map_account_summary_aggregate, merge_perpetual_portfolios, merge_positions_summaries},
        perpetual_mapper::{map_candlesticks, map_perpetual_portfolio, map_perpetuals_data, map_positions},
    },
    rpc::client::HyperCoreClient,
};

fn filter_active_dex(perp_dexs: &[Option<PerpDex>]) -> Vec<(u32, Option<String>)> {
    perp_dexs
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if index == 0 {
                return Some((0, None));
            }
            let dex = entry.as_ref()?;
            if dex.is_active == Some(false) || dex.name.is_empty() {
                return None;
            }
            Some((index as u32, Some(dex.name.clone())))
        })
        .collect()
}

impl<C: Client> HyperCoreClient<C> {
    async fn get_active_dexes(&self) -> Vec<(u32, Option<String>)> {
        self.get_perp_dexs().await.map(|dexs| filter_active_dex(&dexs)).unwrap_or_else(|_| vec![(0, None)])
    }

    async fn for_each_dex<T, F, Fut>(&self, f: F) -> Result<Vec<T>, Box<dyn Error + Sync + Send>>
    where
        F: Fn(u32, Option<String>) -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn Error + Sync + Send>>>,
    {
        let dex_entries = self.get_active_dexes().await;
        let requests: Vec<_> = dex_entries.into_iter().map(|(index, dex)| f(index, dex)).collect();
        try_join_all(requests).await
    }

    async fn fetch_positions_for_dex(&self, address: String, dex: Option<String>) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let (positions, orders) = try_join!(self.get_clearinghouse_state_with_dex(&address, dex.clone()), self.get_open_orders_with_dex(&address, dex))?;
        Ok(map_positions(positions, address, &orders))
    }

    async fn fetch_portfolio_for_dex(&self, address: String, dex: Option<String>) -> Result<(PerpetualPortfolio, AssetPositions), Box<dyn Error + Sync + Send>> {
        let (response, positions) = try_join!(
            self.get_perpetual_portfolio_with_dex(&address, dex.clone()),
            self.get_clearinghouse_state_with_dex(&address, dex)
        )?;
        Ok((map_perpetual_portfolio(response, &positions), positions))
    }
}

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let summaries = self.for_each_dex(|_, dex| self.fetch_positions_for_dex(address.clone(), dex)).await?;
        Ok(merge_positions_summaries(summaries))
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let results = self.for_each_dex(|index, dex| async move {
            let metadata = self.get_metadata_with_dex(dex).await?;
            Ok(map_perpetuals_data(metadata, index))
        }).await?;

        Ok(results.into_iter().flatten().collect())
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
        let results = self.for_each_dex(|_, dex| self.fetch_portfolio_for_dex(address.clone(), dex)).await?;
        let (portfolios, positions): (Vec<_>, Vec<_>) = results.into_iter().unzip();
        let account_summary = Some(map_account_summary_aggregate(&positions));
        Ok(merge_perpetual_portfolios(portfolios, account_summary))
    }

    async fn get_perpetual_referred_addresses(&self) -> Result<Vec<String>, Box<dyn Error + Sync + Send>> {
        let config = HypercoreConfig::default();
        let referral = self.get_referral(&config.builder_address).await?;
        let referral_states = referral.referrer_state.and_then(|s| s.data.referral_states).unwrap_or_default();
        Ok(referral_states.into_iter().map(|r| r.user).collect())
    }
}

#[async_trait]
impl<C: Client> ChainAddressStatus for HyperCoreClient<C> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_active_dex_filters_inactive() {
        let dexs = vec![
            None,
            Some(PerpDex { name: "dex1".to_string(), is_active: Some(true) }),
            Some(PerpDex { name: "dex2".to_string(), is_active: Some(false) }),
            Some(PerpDex { name: "dex3".to_string(), is_active: None }),
        ];

        let entries = filter_active_dex(&dexs);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], (0, None));
        assert_eq!(entries[1], (1, Some("dex1".to_string())));
        assert_eq!(entries[2], (3, Some("dex3".to_string())));
    }

    #[test]
    fn test_filter_active_dex_skips_empty_names() {
        let dexs = vec![
            None,
            Some(PerpDex { name: "".to_string(), is_active: Some(true) }),
        ];

        let entries = filter_active_dex(&dexs);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], (0, None));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_hypercore_test_client};
    use chain_traits::ChainPerpetual;
    use primitives::ChartPeriod;

    #[tokio::test]
    async fn test_hypercore_get_perp_dexs() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let dexs = client.get_perp_dexs().await?;

        assert!(!dexs.is_empty());

        println!("Perp DEXs count: {}", dexs.len());
        for (i, dex) in dexs.iter().enumerate() {
            println!("  DEX {}: {:?}", i, dex.as_ref().map(|d| (&d.name, &d.is_active)));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_positions() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let summary = client.get_positions(TEST_ADDRESS.to_string()).await?;

        println!("Positions count: {}", summary.positions.len());
        println!(
            "Balance: available={}, reserved={}, withdrawable={}",
            summary.balance.available, summary.balance.reserved, summary.balance.withdrawable
        );

        for pos in &summary.positions {
            println!("  {} {:?} size={} leverage={}", pos.perpetual_id, pos.direction, pos.size, pos.leverage);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetuals_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let data = client.get_perpetuals_data().await?;

        assert!(!data.is_empty());

        println!("Perpetuals count: {}", data.len());
        for d in data.iter().take(5) {
            println!(
                "  {} identifier={} price={} leverage={}",
                d.perpetual.name, d.perpetual.identifier, d.perpetual.price, d.perpetual.max_leverage
            );
        }

        let btc = data.iter().find(|d| d.perpetual.name == "BTC");
        assert!(btc.is_some(), "BTC perpetual should exist");
        assert_eq!(btc.unwrap().perpetual.identifier, "0");

        let builder_assets: Vec<_> = data.iter().filter(|d| d.perpetual.identifier.parse::<u32>().unwrap_or(0) >= 100_000).collect();
        println!("Builder DEX assets: {}", builder_assets.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_portfolio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, TEST_ADDRESS.to_string()).await?;

        println!("Perpetual portfolio day: {:?}", portfolio.day.is_some());

        assert!(portfolio.day.is_some() || portfolio.week.is_some() || portfolio.month.is_some() || portfolio.all_time.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_candlesticks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let candlesticks = client.get_perpetual_candlesticks("BTC".to_string(), ChartPeriod::Day).await?;

        println!("Perpetual candlesticks count: {:?}", candlesticks.len());

        assert!(!candlesticks.is_empty());
        Ok(())
    }
}
