use crate::config::swap_config::get_swap_config;
use primitives::swap::SwapQuoteDataType;
pub use primitives::swap::{ApprovalData, SwapData, SwapProviderData, SwapQuote, SwapQuoteData};
pub use swapper::SwapperProvider;

pub type GemApprovalData = ApprovalData;
pub type GemSwapData = SwapData;
pub type GemSwapProviderData = SwapProviderData;
pub type GemSwapQuote = SwapQuote;
pub type GemSwapQuoteData = SwapQuoteData;
pub type GemSwapQuoteDataType = SwapQuoteDataType;

#[uniffi::remote(Record)]
pub struct GemApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub is_unlimited: bool,
}

#[uniffi::remote(Enum)]
pub enum GemSwapQuoteDataType {
    Contract,
    Transfer,
}

#[uniffi::remote(Record)]
pub struct GemSwapData {
    pub quote: GemSwapQuote,
    pub data: GemSwapQuoteData,
}

#[uniffi::remote(Record)]
pub struct GemSwapQuote {
    pub from_address: String,
    pub from_value: String,
    pub to_address: String,
    pub to_value: String,
    pub provider_data: GemSwapProviderData,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
    pub use_max_amount: Option<bool>,
}

#[uniffi::remote(Record)]
pub struct GemSwapQuoteData {
    pub to: String,
    pub data_type: GemSwapQuoteDataType,
    pub value: String,
    pub data: String,
    pub memo: Option<String>,
    pub approval: Option<GemApprovalData>,
    pub gas_limit: Option<String>,
}

#[uniffi::remote(Record)]
pub struct GemSwapProviderData {
    pub provider: SwapperProvider,
    pub name: String,
    pub protocol_name: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SwapPriceImpactType {
    Positive,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct SwapPriceImpact {
    pub percentage: f64,
    pub impact_type: SwapPriceImpactType,
    pub is_high: bool,
}

#[uniffi::export]
pub fn calculate_swap_price_impact(pay_fiat_value: f64, receive_fiat_value: f64) -> Option<SwapPriceImpact> {
    if pay_fiat_value <= 0.0 || receive_fiat_value <= 0.0 || !pay_fiat_value.is_finite() || !receive_fiat_value.is_finite() {
        return None;
    }

    let percentage = ((receive_fiat_value / pay_fiat_value) - 1.0) * 100.0;
    let rounded_percentage = round_to_places(percentage, 2);
    let impact_type = match rounded_percentage {
        value if value > 0.0 => SwapPriceImpactType::Positive,
        value if value >= -1.0 => SwapPriceImpactType::Low,
        value if value >= -5.0 => SwapPriceImpactType::Medium,
        _ => SwapPriceImpactType::High,
    };

    Some(SwapPriceImpact {
        percentage,
        impact_type,
        is_high: rounded_percentage.abs() >= get_swap_config().high_price_impact_percent as f64,
    })
}

fn round_to_places(value: f64, places: i32) -> f64 {
    let factor = 10_f64.powi(places);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::{SwapPriceImpact, SwapPriceImpactType, calculate_swap_price_impact, round_to_places};

    #[test]
    fn test_calculate_swap_price_impact() {
        assert_eq!(calculate_swap_price_impact(0.0, 100.0), None);
        assert_eq!(calculate_swap_price_impact(100.0, 0.0), None);

        assert_eq!(
            calculate_swap_price_impact(100.0, 100.5).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: 0.5,
                impact_type: SwapPriceImpactType::Positive,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 99.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -1.0,
                impact_type: SwapPriceImpactType::Low,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 95.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -5.0,
                impact_type: SwapPriceImpactType::Medium,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 89.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -11.0,
                impact_type: SwapPriceImpactType::High,
                is_high: true,
            })
        );
    }
}
