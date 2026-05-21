use super::{
    constants::MAYAN_MCTP,
    model::{MayanFastMctpQuote, MayanMctpQuote, MayanQuoteCommon, MayanToken},
};
use primitives::asset_constants::{BASE_USDC_TOKEN_ID, ETHEREUM_USDC_TOKEN_ID, SOLANA_USDC_TOKEN_ID, SUI_USDC_TOKEN_ID};

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

impl MayanFastMctpQuote {
    pub fn mock() -> Self {
        Self {
            common: MayanQuoteCommon {
                effective_amount_in64: "1000000".to_string(),
                min_amount_out: serde_json::json!(0.9),
                gas_drop: serde_json::json!(0),
                eta_seconds: 20,
                from_token: MayanToken {
                    contract: SOLANA_USDC_TOKEN_ID.to_string(),
                    w_chain_id: 1,
                    decimals: 6,
                    verified_address: None,
                },
                to_token: MayanToken {
                    contract: BASE_USDC_TOKEN_ID.to_string(),
                    w_chain_id: 30,
                    decimals: 6,
                    verified_address: None,
                },
                from_chain: "solana".to_string(),
                to_chain: "base".to_string(),
                slippage_bps: 50,
                deadline64: Some("1779326929".to_string()),
                referrer_bps: Some(50),
                expected_amount_out_base_units: Some("900000".to_string()),
                expected_amount_out: serde_json::json!(0.9),
                ..Default::default()
            },
            fast_mctp_input_contract: Some(SOLANA_USDC_TOKEN_ID.to_string()),
            fast_mctp_mayan_contract: Some(MAYAN_MCTP.to_string()),
            fast_mctp_min_finality: Some(1000),
            circle_max_fee64: Some("500".to_string()),
            redeem_relayer_fee: Some(serde_json::json!(0.1)),
            redeem_relayer_fee64: Some("1000".to_string()),
            refund_relayer_fee64: Some("2000".to_string()),
            solana_relayer_fee64: Some("179182".to_string()),
            ..Default::default()
        }
    }

    pub fn mock_evm() -> Self {
        let mut route = Self::mock();
        route.common.from_token = MayanToken {
            contract: ETHEREUM_USDC_TOKEN_ID.to_string(),
            w_chain_id: 2,
            decimals: 6,
            verified_address: None,
        };
        route.common.from_chain = "ethereum".to_string();
        route.fast_mctp_input_contract = Some(ETHEREUM_USDC_TOKEN_ID.to_string());
        route
    }
}
