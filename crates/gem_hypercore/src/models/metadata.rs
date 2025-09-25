use primitives::{
    Asset, AssetId, AssetType, Chain,
    perpetual::{Perpetual, PerpetualData, PerpetualMetadata},
    perpetual_provider::PerpetualProvider,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetadata {
    pub funding: String,
    pub open_interest: String,
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub premium: Option<String>,
    pub oracle_px: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub impact_pxs: Option<Vec<String>>,
    pub day_base_vlm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseAsset {
    pub name: String,
    pub sz_decimals: i32,
    pub max_leverage: i32,
    pub only_isolated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreUniverseResponse {
    pub universe: Vec<UniverseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreMetadataResponse(pub HypercoreUniverseResponse, pub Vec<AssetMetadata>);

impl HypercoreMetadataResponse {
    pub fn universe(&self) -> &HypercoreUniverseResponse {
        &self.0
    }

    pub fn asset_metadata(&self) -> &Vec<AssetMetadata> {
        &self.1
    }
}

impl From<HypercoreMetadataResponse> for Vec<PerpetualData> {
    fn from(metadata: HypercoreMetadataResponse) -> Self {
        let universe = metadata.universe();
        let asset_metadata = metadata.asset_metadata();

        universe
            .universe
            .iter()
            .enumerate()
            .map(|(index, universe_asset)| {
                let metadata_item = asset_metadata.get(index);

                let asset_id = AssetId::from(
                    Chain::HyperCore,
                    Some(AssetId::sub_token_id(&["perpetual".to_string(), universe_asset.name.clone()])),
                );

                let current_price = metadata_item
                    .and_then(|m| m.mid_px.as_ref().and_then(|mid| mid.parse().ok()).or_else(|| m.mark_px.parse().ok()))
                    .unwrap_or(0.0);

                let prev_price = metadata_item.and_then(|m| m.prev_day_px.parse().ok()).unwrap_or(0.0);

                let price_change_24h = if prev_price > 0.0 {
                    ((current_price - prev_price) / prev_price) * 100.0
                } else {
                    0.0
                };

                let funding_rate = metadata_item.and_then(|m| m.funding.parse::<f64>().ok()).unwrap_or(0.0) * 100.0;

                let open_interest_coins = metadata_item.and_then(|m| m.open_interest.parse::<f64>().ok()).unwrap_or(0.0);
                let open_interest_usd = open_interest_coins * current_price;

                let perpetual_id = format!("{}_{}", PerpetualProvider::Hypercore.as_ref(), universe_asset.name.clone());
                let perpetual = Perpetual {
                    id: perpetual_id,
                    name: universe_asset.name.clone(),
                    provider: PerpetualProvider::Hypercore,
                    asset_id: asset_id.clone(),
                    identifier: index.to_string(),
                    price: current_price,
                    price_percent_change_24h: price_change_24h,
                    open_interest: open_interest_usd,
                    volume_24h: metadata_item.and_then(|m| m.day_ntl_vlm.parse().ok()).unwrap_or(0.0),
                    funding: funding_rate,
                    leverage: vec![universe_asset.max_leverage as u8],
                };

                let asset = Asset {
                    id: asset_id,
                    chain: Chain::HyperCore,
                    token_id: Some(universe_asset.name.clone()),
                    name: universe_asset.name.clone(),
                    symbol: universe_asset.name.clone(),
                    decimals: universe_asset.sz_decimals,
                    asset_type: AssetType::PERPETUAL,
                };

                let metadata = PerpetualMetadata { is_pinned: false };

                PerpetualData { perpetual, asset, metadata }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypercore_metadata_to_perpetual_data() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![
                UniverseAsset {
                    name: "BTC".to_string(),
                    sz_decimals: 5,
                    max_leverage: 100,
                    only_isolated: None,
                },
                UniverseAsset {
                    name: "ETH".to_string(),
                    sz_decimals: 4,
                    max_leverage: 50,
                    only_isolated: Some(false),
                },
                UniverseAsset {
                    name: "SOL".to_string(),
                    sz_decimals: 3,
                    max_leverage: 20,
                    only_isolated: Some(true),
                },
            ],
        };

        let asset_metadata = vec![
            AssetMetadata {
                funding: "0.0001".to_string(),
                open_interest: "12345.67".to_string(),
                prev_day_px: "60000".to_string(),
                day_ntl_vlm: "1000000".to_string(),
                premium: Some("0.5".to_string()),
                oracle_px: "61000".to_string(),
                mark_px: "61250.5".to_string(),
                mid_px: Some("61125.25".to_string()),
                impact_pxs: Some(vec!["61000".to_string(), "61250".to_string()]),
                day_base_vlm: "500000".to_string(),
            },
            AssetMetadata {
                funding: "-0.0002".to_string(),
                open_interest: "8765.43".to_string(),
                prev_day_px: "3000".to_string(),
                day_ntl_vlm: "800000".to_string(),
                premium: None,
                oracle_px: "3100".to_string(),
                mark_px: "3095.75".to_string(),
                mid_px: None,
                impact_pxs: None,
                day_base_vlm: "400000".to_string(),
            },
            AssetMetadata {
                funding: "0.0003".to_string(),
                open_interest: "5432.10".to_string(),
                prev_day_px: "150".to_string(),
                day_ntl_vlm: "600000".to_string(),
                premium: Some("1.0".to_string()),
                oracle_px: "155".to_string(),
                mark_px: "156.25".to_string(),
                mid_px: Some("155.5".to_string()),
                impact_pxs: Some(vec!["154".to_string(), "157".to_string()]),
                day_base_vlm: "300000".to_string(),
            },
        ];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let perpetual_data: Vec<PerpetualData> = metadata_response.into();

        assert_eq!(perpetual_data.len(), 3);

        // Test BTC perpetual data
        let btc_data = &perpetual_data[0];
        assert_eq!(btc_data.perpetual.id, "hypercore_BTC");
        assert_eq!(btc_data.perpetual.name, "BTC");
        assert_eq!(btc_data.perpetual.provider, PerpetualProvider::Hypercore);
        assert_eq!(btc_data.perpetual.asset_id.to_string(), "hypercore_perpetual::BTC");
        assert_eq!(btc_data.perpetual.identifier, "0");
        assert_eq!(btc_data.perpetual.price, 61125.25);

        // Calculate price change: ((61125.25 - 60000) / 60000) * 100 = 1.8754...
        let expected_change = ((61125.25 - 60000.0) / 60000.0) * 100.0;
        assert!((btc_data.perpetual.price_percent_change_24h - expected_change).abs() < 0.0001);

        // Calculate open interest: 12345.67 * 61125.25 = 754,819,827.5975
        let expected_oi = 12345.67 * 61125.25;
        assert!((btc_data.perpetual.open_interest - expected_oi).abs() < 1.0);

        assert_eq!(btc_data.perpetual.volume_24h, 1000000.0);
        assert_eq!(btc_data.perpetual.funding, 0.01);
        assert_eq!(btc_data.perpetual.leverage, vec![100]);

        assert_eq!(btc_data.asset.id.to_string(), "hypercore_perpetual::BTC");
        assert_eq!(btc_data.asset.name, "BTC");
        assert_eq!(btc_data.asset.symbol, "BTC");
        assert_eq!(btc_data.asset.decimals, 5);

        assert!(!btc_data.metadata.is_pinned);

        // Test ETH perpetual data
        let eth_data = &perpetual_data[1];
        assert_eq!(eth_data.perpetual.id, "hypercore_ETH");
        assert_eq!(eth_data.perpetual.price, 3095.75);
        assert_eq!(eth_data.perpetual.funding, -0.02);
        assert_eq!(eth_data.perpetual.leverage, vec![50]);
        assert_eq!(eth_data.asset.decimals, 4);

        // Test SOL perpetual data
        let sol_data = &perpetual_data[2];
        assert_eq!(sol_data.perpetual.id, "hypercore_SOL");
        assert_eq!(sol_data.perpetual.price, 155.5);
        assert_eq!(sol_data.perpetual.funding, 0.03);
        assert_eq!(sol_data.perpetual.leverage, vec![20]);
        assert_eq!(sol_data.asset.decimals, 3);
    }

    #[test]
    fn test_price_calculation_with_missing_mid_px() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "TEST".to_string(),
                sz_decimals: 2,
                max_leverage: 10,
                only_isolated: None,
            }],
        };

        let asset_metadata = vec![AssetMetadata {
            funding: "0.0001".to_string(),
            open_interest: "1000".to_string(),
            prev_day_px: "100".to_string(),
            day_ntl_vlm: "50000".to_string(),
            premium: None,
            oracle_px: "105".to_string(),
            mark_px: "102.5".to_string(),
            mid_px: None,
            impact_pxs: None,
            day_base_vlm: "25000".to_string(),
        }];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let perpetual_data: Vec<PerpetualData> = metadata_response.into();

        assert_eq!(perpetual_data.len(), 1);

        let test_data = &perpetual_data[0];
        assert_eq!(test_data.perpetual.price, 102.5);
        assert_eq!(test_data.perpetual.price_percent_change_24h, 2.5);
    }

    #[test]
    fn test_price_change_calculation() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "TEST".to_string(),
                sz_decimals: 2,
                max_leverage: 10,
                only_isolated: None,
            }],
        };

        let test_cases = vec![
            // (current_price, prev_price, expected_change_percent)
            ("110", "100", 10.0),
            ("90", "100", -10.0),
            ("100", "100", 0.0),
            ("0", "100", -100.0),
        ];

        for (current, prev, expected) in test_cases {
            let asset_metadata = vec![AssetMetadata {
                funding: "0".to_string(),
                open_interest: "1000".to_string(),
                prev_day_px: prev.to_string(),
                day_ntl_vlm: "50000".to_string(),
                premium: None,
                oracle_px: "105".to_string(),
                mark_px: current.to_string(),
                mid_px: None,
                impact_pxs: None,
                day_base_vlm: "25000".to_string(),
            }];

            let metadata_response = HypercoreMetadataResponse(universe_response.clone(), asset_metadata);
            let perpetual_data: Vec<PerpetualData> = metadata_response.into();

            assert_eq!(perpetual_data[0].perpetual.price_percent_change_24h, expected);
        }
    }

    #[test]
    fn test_asset_id_subtoken_pattern() {
        let universe_response = HypercoreUniverseResponse {
            universe: vec![UniverseAsset {
                name: "BTC".to_string(),
                sz_decimals: 5,
                max_leverage: 100,
                only_isolated: None,
            }],
        };

        let asset_metadata = vec![AssetMetadata {
            funding: "0.0001".to_string(),
            open_interest: "1000".to_string(),
            prev_day_px: "60000".to_string(),
            day_ntl_vlm: "1000000".to_string(),
            premium: None,
            oracle_px: "61000".to_string(),
            mark_px: "61250.5".to_string(),
            mid_px: None,
            impact_pxs: None,
            day_base_vlm: "500000".to_string(),
        }];

        let metadata_response = HypercoreMetadataResponse(universe_response, asset_metadata);
        let perpetual_data: Vec<PerpetualData> = metadata_response.into();

        let btc_data = &perpetual_data[0];

        assert_eq!(btc_data.perpetual.asset_id.chain, primitives::Chain::HyperCore);
        assert_eq!(btc_data.perpetual.asset_id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(btc_data.perpetual.asset_id.to_string(), "hypercore_perpetual::BTC");

        assert_eq!(btc_data.asset.id.chain, primitives::Chain::HyperCore);
        assert_eq!(btc_data.asset.id.token_id, Some("perpetual::BTC".to_string()));
        assert_eq!(btc_data.asset.id.to_string(), "hypercore_perpetual::BTC");
    }
}
