use serde::{de::Error as SerdeError, Deserialize, Serialize};
use typeshare::typeshare;

use super::referral::ReferralInfo;
use crate::{AssetId, Chain};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct QuoteAsset {
    #[serde(skip)]
    pub id: AssetId,
    pub asset_id: String,
    pub symbol: String,
    pub decimals: u32,
}

impl<'de> Deserialize<'de> for QuoteAsset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            AssetId,
            Symbol,
            Decimals,
        }

        struct QuoteAssetVisitor;

        impl<'de> serde::de::Visitor<'de> for QuoteAssetVisitor {
            type Value = QuoteAsset;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct QuoteAsset")
            }

            fn visit_map<V>(self, mut map: V) -> Result<QuoteAsset, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut asset_id_str: Option<String> = None;
                let mut symbol: Option<String> = None;
                let mut decimals: Option<u32> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AssetId => {
                            if asset_id_str.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id_str = Some(map.next_value()?);
                        }
                        Field::Symbol => {
                            if symbol.is_some() {
                                return Err(serde::de::Error::duplicate_field("symbol"));
                            }
                            symbol = Some(map.next_value()?);
                        }
                        Field::Decimals => {
                            if decimals.is_some() {
                                return Err(serde::de::Error::duplicate_field("decimals"));
                            }
                            decimals = Some(map.next_value()?);
                        }
                    }
                }

                let asset_id_str = asset_id_str.ok_or_else(|| SerdeError::missing_field("assetId"))?;
                let symbol = symbol.ok_or_else(|| SerdeError::missing_field("symbol"))?;
                let decimals = decimals.ok_or_else(|| SerdeError::missing_field("decimals"))?;
                let id = AssetId::new(&asset_id_str).ok_or(SerdeError::custom("Invalid AssetId"))?;

                Ok(QuoteAsset {
                    id,
                    asset_id: asset_id_str,
                    symbol,
                    decimals,
                })
            }
        }

        const FIELDS: &[&str] = &["assetId", "symbol", "decimals"];
        deserializer.deserialize_struct("QuoteAsset", FIELDS, QuoteAssetVisitor)
    }
}

impl QuoteAsset {
    pub fn is_native(&self) -> bool {
        self.id.is_native()
    }

    pub fn chain(&self) -> Chain {
        self.id.chain
    }
}

impl From<AssetId> for QuoteAsset {
    fn from(id: AssetId) -> Self {
        Self {
            id: id.clone(),
            asset_id: id.to_string(),
            symbol: String::new(),
            decimals: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteRequest {
    pub from_address: String,
    pub to_address: String,
    pub from_asset: QuoteAsset,
    pub to_asset: QuoteAsset,
    pub from_value: String,
    pub referral: ReferralInfo,
    pub slippage_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Quote {
    pub quote: QuoteRequest,
    pub output_value: String,
    pub output_min_value: String,
    pub route_data: serde_json::Value,
    pub eta_in_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct QuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub limit: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AssetId;

    #[test]
    fn test_deserialize_quote_asset() {
        let asset_id = "ethereum_0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
        let json_data = format!(
            r#"{{
            "assetId": "{asset_id}",
            "symbol": "WETH",
            "decimals": 18
        }}"#
        );

        let result: Result<QuoteAsset, _> = serde_json::from_str(json_data.as_str());
        assert!(result.is_ok());

        let quote_asset = result.unwrap();
        assert_eq!(quote_asset.asset_id, asset_id);
        assert_eq!(quote_asset.symbol, "WETH");
        assert_eq!(quote_asset.decimals, 18);

        // Verify the AssetId was correctly constructed using AssetId::new internally
        let expected_id = AssetId::new(asset_id).unwrap();
        assert_eq!(quote_asset.id, expected_id);
    }
}
