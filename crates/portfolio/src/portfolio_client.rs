use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;

use number_formatter::BigNumberFormatter;
use primitives::{ChartPeriod, ChartValue, ChartValuePercentage, PortfolioAllocation, PortfolioAsset, PortfolioAssets};
use storage::{AssetsRepository, ChartsRepository, Database, PricesRepository};

pub struct PortfolioClient {
    database: Database,
}

struct ResolvedAsset {
    asset: PortfolioAsset,
    balance: f64,
    price_id: String,
    current_value: f64,
}

impl PortfolioClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_portfolio_charts(&self, assets: Vec<PortfolioAsset>, period: ChartPeriod) -> Result<PortfolioAssets, Box<dyn Error + Send + Sync>> {
        let assets: Vec<ResolvedAsset> = assets.into_iter().filter_map(|input| self.resolve_asset(input)).collect();
        let chart_data = self.get_chart_values(&assets, &period);
        Ok(Self::build_portfolio(assets, chart_data))
    }

    fn get_chart_values(&self, assets: &[ResolvedAsset], period: &ChartPeriod) -> BTreeMap<i64, f64> {
        assets
            .iter()
            .flat_map(|r| {
                self.database
                    .charts()
                    .ok()
                    .and_then(|mut db| db.get_charts(&r.price_id, period).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(ts, price)| (ts.and_utc().timestamp(), r.balance * price))
            })
            .fold(BTreeMap::new(), |mut acc, (ts, value)| {
                *acc.entry(ts).or_default() += value;
                acc
            })
    }

    fn build_portfolio(assets: Vec<ResolvedAsset>, chart_data: BTreeMap<i64, f64>) -> PortfolioAssets {
        let values: Vec<ChartValue> = chart_data
            .into_iter()
            .map(|(ts, value)| ChartValue {
                timestamp: ts as i32,
                value: value as f32,
            })
            .collect();

        let cmp = |a: &&ChartValue, b: &&ChartValue| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal);
        let all_time_high = values.iter().max_by(cmp).cloned();
        let all_time_low = values.iter().min_by(cmp).cloned();

        let total_value: f64 = assets.iter().map(|r| r.current_value).sum();
        let total_value_f32 = total_value as f32;

        let to_percentage = |cv: &ChartValue| ChartValuePercentage {
            date: chrono::DateTime::from_timestamp(cv.timestamp as i64, 0).unwrap_or_default(),
            value: cv.value,
            percentage: if total_value_f32 > 0.0 {
                (cv.value - total_value_f32) / total_value_f32 * 100.0
            } else {
                0.0
            },
        };

        let allocation: Vec<PortfolioAllocation> = assets
            .into_iter()
            .map(|r| PortfolioAllocation {
                asset_id: r.asset.asset_id,
                value: r.current_value as f32,
                percentage: if total_value > 0.0 { (r.current_value / total_value) as f32 } else { 0.0 },
            })
            .collect();

        PortfolioAssets {
            total_value: total_value_f32,
            values,
            all_time_high: all_time_high.as_ref().map(to_percentage),
            all_time_low: all_time_low.as_ref().map(to_percentage),
            allocation,
        }
    }

    fn resolve_asset(&self, input: PortfolioAsset) -> Option<ResolvedAsset> {
        let asset_id = input.asset_id.to_string();
        let asset = self.database.assets().ok()?.get_asset(&asset_id).ok()?;
        let balance = BigNumberFormatter::value_as_f64(&input.value, asset.decimals as u32).ok()?;
        let key = self.database.prices().ok()?.get_primary_price_key(&asset_id).ok()?;
        let price_id = key.id();
        let price = self.database.prices().ok()?.get_price_by_id(&price_id).map(|p| p.price).unwrap_or_default();

        Some(ResolvedAsset {
            asset: input,
            balance,
            price_id,
            current_value: balance * price,
        })
    }
}
