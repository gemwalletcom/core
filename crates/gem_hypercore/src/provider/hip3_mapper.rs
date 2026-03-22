use std::collections::BTreeMap;

use primitives::{
    chart::ChartDateValue,
    perpetual::{PerpetualBalance, PerpetualPositionsSummary},
    portfolio::{PerpetualAccountSummary, PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

use crate::models::position::AssetPositions;

const HIP3_PERP_ASSET_OFFSET: u32 = 100_000;
const HIP3_PERP_ASSET_STRIDE: u32 = 10_000;

pub fn perp_asset_index(perp_dex_index: u32, meta_index: u32) -> u32 {
    if perp_dex_index == 0 {
        meta_index
    } else {
        HIP3_PERP_ASSET_OFFSET + perp_dex_index * HIP3_PERP_ASSET_STRIDE + meta_index
    }
}

pub fn format_display_name(name: &str) -> String {
    match name.split_once(':') {
        Some((dex, symbol)) => format!("{symbol} ({dex})"),
        None => name.to_string(),
    }
}

pub fn map_account_summary_aggregate(positions: &[AssetPositions]) -> PerpetualAccountSummary {
    let account_value: f64 = positions.iter().map(|p| p.margin_summary.account_value.parse().unwrap_or(0.0)).sum();
    let total_ntl_pos: f64 = positions.iter().map(|p| p.margin_summary.total_ntl_pos.parse().unwrap_or(0.0)).sum();
    let total_margin_used: f64 = positions.iter().map(|p| p.margin_summary.total_margin_used.parse().unwrap_or(0.0)).sum();
    let unrealized_pnl: f64 = positions
        .iter()
        .flat_map(|p| &p.asset_positions)
        .map(|p| p.position.unrealized_pnl.parse().unwrap_or(0.0))
        .sum();

    let account_leverage = if account_value > 0.0 { total_ntl_pos / account_value } else { 0.0 };
    let margin_usage = if account_value > 0.0 { total_margin_used / account_value } else { 0.0 };

    PerpetualAccountSummary {
        account_value,
        account_leverage,
        margin_usage,
        unrealized_pnl,
    }
}

pub fn merge_positions_summaries(summaries: Vec<PerpetualPositionsSummary>) -> PerpetualPositionsSummary {
    let (positions, balance) = summaries.into_iter().fold(
        (Vec::new(), PerpetualBalance { available: 0.0, reserved: 0.0, withdrawable: 0.0 }),
        |(mut acc_pos, mut acc_bal), summary| {
            acc_pos.extend(summary.positions);
            acc_bal.available += summary.balance.available;
            acc_bal.reserved += summary.balance.reserved;
            acc_bal.withdrawable += summary.balance.withdrawable;
            (acc_pos, acc_bal)
        },
    );
    PerpetualPositionsSummary { positions, balance }
}

pub fn merge_perpetual_portfolios(portfolios: Vec<PerpetualPortfolio>, account_summary: Option<PerpetualAccountSummary>) -> PerpetualPortfolio {
    let mut day = Vec::new();
    let mut week = Vec::new();
    let mut month = Vec::new();
    let mut all_time = Vec::new();

    for portfolio in portfolios {
        day.extend(portfolio.day);
        week.extend(portfolio.week);
        month.extend(portfolio.month);
        all_time.extend(portfolio.all_time);
    }

    PerpetualPortfolio {
        day: merge_portfolio_timeframes(day),
        week: merge_portfolio_timeframes(week),
        month: merge_portfolio_timeframes(month),
        all_time: merge_portfolio_timeframes(all_time),
        account_summary,
    }
}

fn merge_portfolio_timeframes(values: Vec<PerpetualPortfolioTimeframeData>) -> Option<PerpetualPortfolioTimeframeData> {
    if values.is_empty() {
        return None;
    }

    let mut account_value_histories = Vec::new();
    let mut pnl_histories = Vec::new();
    let mut volume = 0.0;

    for value in values {
        account_value_histories.push(value.account_value_history);
        pnl_histories.push(value.pnl_history);
        volume += value.volume;
    }

    Some(PerpetualPortfolioTimeframeData {
        account_value_history: merge_chart_histories(account_value_histories),
        pnl_history: merge_chart_histories(pnl_histories),
        volume,
    })
}

fn merge_chart_histories(values: Vec<Vec<ChartDateValue>>) -> Vec<ChartDateValue> {
    let mut grouped = BTreeMap::new();
    for history in values {
        for point in history {
            let entry = grouped.entry(point.date).or_insert(0.0);
            *entry += point.value;
        }
    }

    grouped.into_iter().map(|(date, value)| ChartDateValue { date, value }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::portfolio::PerpetualAccountSummary;

    #[test]
    fn test_perp_asset_index() {
        assert_eq!(perp_asset_index(0, 0), 0);
        assert_eq!(perp_asset_index(0, 5), 5);
        assert_eq!(perp_asset_index(1, 0), 110_000);
        assert_eq!(perp_asset_index(1, 3), 110_003);
        assert_eq!(perp_asset_index(2, 0), 120_000);
        assert_eq!(perp_asset_index(2, 7), 120_007);
    }

    #[test]
    fn test_format_display_name() {
        assert_eq!(format_display_name("xyz:GOLD"), "GOLD (xyz)");
        assert_eq!(format_display_name("BTC"), "BTC");
    }

    #[test]
    fn test_map_account_summary_aggregate() {
        let positions = vec![AssetPositions::mock(), AssetPositions::mock()];
        let summary = map_account_summary_aggregate(&positions);

        assert_eq!(summary.account_value, 20000.0);
        assert_eq!(summary.account_leverage, 0.5);
        assert_eq!(summary.margin_usage, 0.2);
        assert_eq!(summary.unrealized_pnl, 0.0);
    }

    #[test]
    fn test_merge_chart_histories() {
        use chrono::{TimeZone, Utc};

        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let d3 = Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap();

        let histories = vec![
            vec![ChartDateValue { date: d1, value: 100.0 }, ChartDateValue { date: d2, value: 200.0 }],
            vec![ChartDateValue { date: d1, value: 50.0 }, ChartDateValue { date: d3, value: 300.0 }],
        ];

        let merged = merge_chart_histories(histories);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].value, 150.0);
        assert_eq!(merged[1].value, 200.0);
        assert_eq!(merged[2].value, 300.0);
    }

    #[test]
    fn test_merge_perpetual_portfolios() {
        use chrono::{TimeZone, Utc};

        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let portfolios = vec![
            PerpetualPortfolio::mock_with_day(d1, 100.0, 10.0, 500.0),
            PerpetualPortfolio::mock_with_day(d1, 200.0, 20.0, 300.0),
        ];
        let summary = PerpetualAccountSummary { account_value: 1000.0, account_leverage: 2.0, margin_usage: 0.5, unrealized_pnl: 30.0 };

        let merged = merge_perpetual_portfolios(portfolios, Some(summary));

        let day = merged.day.unwrap();
        assert_eq!(day.volume, 800.0);
        assert_eq!(day.account_value_history.len(), 1);
        assert_eq!(day.account_value_history[0].value, 300.0);
        assert_eq!(day.pnl_history[0].value, 30.0);
        assert!(merged.week.is_none());
        assert_eq!(merged.account_summary.unwrap().account_value, 1000.0);
    }

    #[test]
    fn test_merge_portfolio_timeframes_empty() {
        let result = merge_portfolio_timeframes(vec![]);
        assert!(result.is_none());
    }
}
