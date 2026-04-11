use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use futures::{future::try_join_all, try_join};
use gem_client::Client;
use primitives::{
    ChartPeriod,
    chart::ChartCandleStick,
    perpetual::{PerpetualBalance, PerpetualData, PerpetualPositionsSummary},
    portfolio::PerpetualPortfolio,
};

use crate::{
    config::HypercoreConfig,
    models::{order::OpenOrder, perp_dex::PerpDex, position::AssetPositions},
    provider::perpetual_mapper::{map_account_summary_aggregate, map_candlesticks, map_perpetual_portfolio, map_perpetuals_data, map_positions, merge_perpetual_portfolios},
    rpc::client::HyperCoreClient,
};

fn filter_active_dex(perp_dexs: &[Option<PerpDex>], enabled_hip3_markets: &[String]) -> Vec<(u32, Option<String>)> {
    perp_dexs
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if index == 0 {
                return Some((0, None));
            }
            let dex = entry.as_ref()?;
            if dex.is_active == Some(false) {
                return None;
            }
            if dex.name.is_empty() {
                return None;
            }
            if enabled_hip3_markets.is_empty() {
                return None;
            }
            if !enabled_hip3_markets.iter().any(|market| market == &dex.name) {
                return None;
            }
            Some((index as u32, Some(dex.name.clone())))
        })
        .collect()
}

pub fn candle_interval(period: &ChartPeriod) -> &'static str {
    match period {
        ChartPeriod::Hour => "1m",
        ChartPeriod::Day => "30m",
        ChartPeriod::Week => "4h",
        ChartPeriod::Month => "12h",
        ChartPeriod::Year => "1w",
        ChartPeriod::All => "1M",
    }
}

impl<C: Client> HyperCoreClient<C> {
    async fn get_active_dex_entries(&self) -> Vec<(u32, Option<String>)> {
        if self.config.enabled_hip3_markets.is_empty() {
            return vec![(0, None)];
        }

        self.get_perp_dexs()
            .await
            .map(|dexs| filter_active_dex(&dexs, &self.config.enabled_hip3_markets))
            .unwrap_or_else(|_| vec![(0, None)])
    }

    async fn get_positions_for_dex(&self, address: String, dex: Option<String>) -> Result<AssetPositions, Box<dyn Error + Sync + Send>> {
        match dex.as_deref() {
            Some(dex) => self.get_clearinghouse_state_with_dex(&address, dex).await,
            None => self.get_clearinghouse_state(&address).await,
        }
    }

    async fn get_open_orders_for_dex(&self, address: String, dex: Option<String>) -> Result<Vec<OpenOrder>, Box<dyn Error + Sync + Send>> {
        match dex.as_deref() {
            Some(dex) => self.get_open_orders_with_dex(&address, dex).await,
            None => self.get_open_orders(&address).await,
        }
    }

    async fn get_portfolio_for_dex(&self, address: String, dex: Option<String>) -> Result<(PerpetualPortfolio, AssetPositions), Box<dyn Error + Sync + Send>> {
        let (response, positions) = match dex.as_deref() {
            Some(dex) => try_join!(self.get_perpetual_portfolio_with_dex(&address, dex), self.get_clearinghouse_state_with_dex(&address, dex))?,
            None => try_join!(self.get_perpetual_portfolio(&address), self.get_clearinghouse_state(&address))?,
        };
        Ok((map_perpetual_portfolio(response, &positions), positions))
    }
}

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let dex_entries = self.get_active_dex_entries().await;
        let summaries = try_join_all(dex_entries.into_iter().map(|(_, dex)| {
            let address = address.clone();
            async move {
                let positions = self.get_positions_for_dex(address.clone(), dex.clone()).await?;
                let orders = if positions.asset_positions.is_empty() {
                    Vec::new()
                } else {
                    self.get_open_orders_for_dex(address.clone(), dex).await?
                };
                Ok::<_, Box<dyn Error + Sync + Send>>(map_positions(positions, address, &orders))
            }
        }))
        .await?;

        let (positions, balance) = summaries.into_iter().fold(
            (
                Vec::new(),
                PerpetualBalance {
                    available: 0.0,
                    reserved: 0.0,
                    withdrawable: 0.0,
                },
            ),
            |(mut acc_pos, mut acc_bal), summary| {
                acc_pos.extend(summary.positions);
                acc_bal.available += summary.balance.available;
                acc_bal.reserved += summary.balance.reserved;
                acc_bal.withdrawable += summary.balance.withdrawable;
                (acc_pos, acc_bal)
            },
        );
        Ok(PerpetualPositionsSummary { positions, balance })
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let dex_entries = self.get_active_dex_entries().await;
        let requests: Vec<_> = dex_entries
            .iter()
            .map(|(_, dex)| async move {
                match dex.as_deref() {
                    Some(dex) => self.get_metadata_with_dex(dex).await,
                    None => self.get_metadata().await,
                }
            })
            .collect();
        let metadata = try_join_all(requests).await?;

        Ok(dex_entries.iter().zip(metadata).flat_map(|((index, _), meta)| map_perpetuals_data(meta, *index)).collect())
    }

    async fn get_perpetual_candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        let interval = candle_interval(&period);

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
        let dex_entries = self.get_active_dex_entries().await;
        let requests: Vec<_> = dex_entries.iter().map(|(_, dex)| self.get_portfolio_for_dex(address.clone(), dex.clone())).collect();
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
    use std::sync::{Arc, Mutex};

    use gem_client::{ClientError, testkit::MockClient};
    use primitives::testkit::json::load_testdata;
    use serde_json::Value;

    use crate::rpc::client::InMemoryPreferences;

    #[test]
    fn test_filter_active_dex_filters_inactive() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "dex1".to_string(),
                is_active: Some(true),
            }),
            Some(PerpDex {
                name: "dex2".to_string(),
                is_active: Some(false),
            }),
            Some(PerpDex {
                name: "dex3".to_string(),
                is_active: None,
            }),
        ];

        let enabled_hip3_markets = vec!["dex1".to_string(), "dex3".to_string()];
        let entries = filter_active_dex(&dexs, &enabled_hip3_markets);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], (0, None));
        assert_eq!(entries[1], (1, Some("dex1".to_string())));
        assert_eq!(entries[2], (3, Some("dex3".to_string())));
    }

    #[test]
    fn test_filter_active_dex_skips_empty_names() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "".to_string(),
                is_active: Some(true),
            }),
        ];

        let enabled_hip3_markets = vec!["dex1".to_string()];
        let entries = filter_active_dex(&dexs, &enabled_hip3_markets);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], (0, None));
    }

    #[test]
    fn test_filter_active_dex_limits_to_enabled_market() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "dex1".to_string(),
                is_active: Some(true),
            }),
            Some(PerpDex {
                name: "xyz".to_string(),
                is_active: Some(true),
            }),
        ];

        let enabled_hip3_markets = vec!["xyz".to_string()];
        let entries = filter_active_dex(&dexs, &enabled_hip3_markets);
        assert_eq!(entries, vec![(0, None), (2, Some("xyz".to_string()))]);
    }

    #[test]
    fn test_filter_active_dex_skips_hip3_when_enabled_markets_empty() {
        let dexs = vec![
            None,
            Some(PerpDex {
                name: "xyz".to_string(),
                is_active: Some(true),
            }),
        ];

        let enabled_hip3_markets = Vec::new();
        let entries = filter_active_dex(&dexs, &enabled_hip3_markets);
        assert_eq!(entries, vec![(0, None)]);
    }

    #[tokio::test]
    async fn test_get_positions_skips_open_orders_for_dexes_without_positions() {
        let perp_dexs_request: Value = load_testdata("perpetual_positions_request_perp_dexs.json");
        let clearinghouse_state_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state.json");
        let clearinghouse_state_dex1_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state_dex1.json");
        let clearinghouse_state_dex2_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state_dex2.json");
        let open_orders_request: Value = load_testdata("perpetual_positions_request_open_orders.json");
        let open_orders_dex1_request: Value = load_testdata("perpetual_positions_request_open_orders_dex1.json");
        let open_orders_dex2_request: Value = load_testdata("perpetual_positions_request_open_orders_dex2.json");

        let responses = Arc::new(vec![
            (perp_dexs_request, include_bytes!("../../testdata/perpetual_positions_response_perp_dexs.json").to_vec()),
            (
                clearinghouse_state_request,
                include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
            ),
            (
                clearinghouse_state_dex1_request,
                include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_dex1.json").to_vec(),
            ),
            (
                clearinghouse_state_dex2_request,
                include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state_dex2.json").to_vec(),
            ),
            (
                open_orders_request.clone(),
                include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec(),
            ),
            (
                open_orders_dex1_request.clone(),
                include_bytes!("../../testdata/perpetual_positions_response_open_orders_dex1.json").to_vec(),
            ),
        ]);
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let responses_clone = Arc::clone(&responses);
        let seen_requests_clone = Arc::clone(&seen_requests);
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/info");

            let request: Value = serde_json::from_slice(body).unwrap();
            seen_requests_clone.lock().unwrap().push(request.clone());

            responses_clone
                .iter()
                .find(|(expected_request, _)| *expected_request == request)
                .map(|(_, response)| response.clone())
                .ok_or_else(|| ClientError::Http { status: 404, body: body.to_vec() })
        });

        let preferences = Arc::new(InMemoryPreferences::new());
        let secure_preferences = Arc::new(InMemoryPreferences::new());
        let mut client = HyperCoreClient::new_with_preferences(client, preferences, secure_preferences);
        client.config.enabled_hip3_markets = vec!["dex1".to_string(), "dex2".to_string()];

        let summary = client.get_positions("0x123".to_string()).await.unwrap();
        let seen_requests = seen_requests.lock().unwrap().clone();

        let btc = summary.positions.iter().find(|position| position.perpetual_id == "hypercore_BTC").unwrap();
        let eth = summary.positions.iter().find(|position| position.perpetual_id == "hypercore_ETH").unwrap();

        assert!(seen_requests.contains(&open_orders_request));
        assert!(seen_requests.contains(&open_orders_dex1_request));
        assert!(!seen_requests.contains(&open_orders_dex2_request));
        assert_eq!(summary.positions.len(), 2);
        assert_eq!(btc.take_profit.as_ref().map(|order| order.price), Some(110.0));
        assert_eq!(eth.stop_loss.as_ref().map(|order| order.price), Some(90.0));
    }

    #[tokio::test]
    async fn test_get_positions_skips_hip3_requests_when_enabled_markets_empty() {
        let perp_dexs_request: Value = load_testdata("perpetual_positions_request_perp_dexs.json");
        let clearinghouse_state_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state.json");
        let clearinghouse_state_dex1_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state_dex1.json");
        let clearinghouse_state_dex2_request: Value = load_testdata("perpetual_positions_request_clearinghouse_state_dex2.json");
        let open_orders_request: Value = load_testdata("perpetual_positions_request_open_orders.json");
        let open_orders_dex1_request: Value = load_testdata("perpetual_positions_request_open_orders_dex1.json");
        let open_orders_dex2_request: Value = load_testdata("perpetual_positions_request_open_orders_dex2.json");

        let responses = Arc::new(vec![
            (
                clearinghouse_state_request.clone(),
                include_bytes!("../../testdata/perpetual_positions_response_clearinghouse_state.json").to_vec(),
            ),
            (
                open_orders_request.clone(),
                include_bytes!("../../testdata/perpetual_positions_response_open_orders.json").to_vec(),
            ),
        ]);
        let seen_requests = Arc::new(Mutex::new(Vec::new()));
        let responses_clone = Arc::clone(&responses);
        let seen_requests_clone = Arc::clone(&seen_requests);
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/info");

            let request: Value = serde_json::from_slice(body).unwrap();
            seen_requests_clone.lock().unwrap().push(request.clone());

            responses_clone
                .iter()
                .find(|(expected_request, _)| *expected_request == request)
                .map(|(_, response)| response.clone())
                .ok_or_else(|| ClientError::Http { status: 404, body: body.to_vec() })
        });

        let preferences = Arc::new(InMemoryPreferences::new());
        let secure_preferences = Arc::new(InMemoryPreferences::new());
        let client = HyperCoreClient::new_with_preferences(client, preferences, secure_preferences);

        let summary = client.get_positions("0x123".to_string()).await.unwrap();
        let seen_requests = seen_requests.lock().unwrap().clone();

        assert_eq!(summary.positions.len(), 1);
        assert!(!seen_requests.contains(&perp_dexs_request));
        assert!(seen_requests.contains(&clearinghouse_state_request));
        assert!(seen_requests.contains(&open_orders_request));
        assert!(!seen_requests.contains(&clearinghouse_state_dex1_request));
        assert!(!seen_requests.contains(&clearinghouse_state_dex2_request));
        assert!(!seen_requests.contains(&open_orders_dex1_request));
        assert!(!seen_requests.contains(&open_orders_dex2_request));
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
