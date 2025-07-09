use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_bigdecimal_from_f64, deserialize_option_bigdecimal_from_f64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankChain {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankTokenItem {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub logo_url: Option<String>,
    pub protocol_id: Option<String>,
    #[serde(deserialize_with = "deserialize_option_bigdecimal_from_f64", default)]
    pub price: Option<BigDecimal>,
    #[serde(deserialize_with = "deserialize_bigdecimal_from_f64")]
    pub amount: BigDecimal,
    pub is_verified: bool,
    pub time_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankProtocol {
    pub id: String,
    pub chain: String,
    pub name: String,
    pub site_url: Option<String>,
    pub logo_url: Option<String>,
    pub has_supported_portfolio: bool,
    #[serde(deserialize_with = "deserialize_bigdecimal_from_f64", default)]
    pub tvl: BigDecimal,
    pub portfolio_item_list: Vec<DeBankPortfolioItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioItem {
    pub stats: DeBankPortfolioStats,
    pub asset_token_list: Option<Vec<DeBankTokenItem>>,
    pub name: String,
    pub detail_types: Vec<String>,
    pub detail: DeBankPortfolioDetail,
    pub pool: Option<DeBankPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioStats {
    #[serde(deserialize_with = "deserialize_bigdecimal_from_f64")]
    pub asset_usd_value: BigDecimal,
    #[serde(deserialize_with = "deserialize_bigdecimal_from_f64")]
    pub debt_usd_value: BigDecimal,
    #[serde(deserialize_with = "deserialize_bigdecimal_from_f64")]
    pub net_usd_value: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPortfolioDetail {
    pub supply_token_list: Option<Vec<DeBankTokenItem>>,
    pub reward_token_list: Option<Vec<DeBankTokenItem>>,
    pub borrow_token_list: Option<Vec<DeBankTokenItem>>,
    #[serde(deserialize_with = "deserialize_option_bigdecimal_from_f64", default)]
    pub health_rate: Option<BigDecimal>,
    pub description: Option<String>,
    pub unlock_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeBankPool {
    pub id: String,
    pub chain: String,
    pub project_id: String,
    pub controller: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_protocol_list() {
        let json = include_str!("./protocol_list.json");
        let protocol_list: Vec<DeBankProtocol> = serde_json::from_str(json).unwrap();

        println!("{protocol_list:?}");
    }
}
