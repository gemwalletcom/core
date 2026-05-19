use super::model::{MayanMctpQuote, MayanQuoteCommon, MayanToken};
use primitives::asset_constants::SUI_USDC_TOKEN_ID;

impl MayanMctpQuote {
    pub fn mock() -> Self {
        Self {
            common: MayanQuoteCommon {
                effective_amount_in64: "1000000".to_string(),
                from_token: MayanToken {
                    contract: SUI_USDC_TOKEN_ID.to_string(),
                    w_chain_id: 21,
                    decimals: 6,
                    verified_address: Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
                },
                from_chain: "sui".to_string(),
                ..Default::default()
            },
            mctp_input_contract: Some(SUI_USDC_TOKEN_ID.to_string()),
            mctp_verified_input_address: Some("0x0000000000000000000000000000000000000000000000000000000000000002".to_string()),
            mctp_input_treasury: Some("0x0000000000000000000000000000000000000000000000000000000000000003".to_string()),
            ..Default::default()
        }
    }
}
