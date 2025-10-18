use crate::{SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, config::get_swap_config};

use super::{Options, QuoteRequest, SwapperMode};

pub fn mock_quote(from_asset: SwapperQuoteAsset, to_asset: SwapperQuoteAsset) -> QuoteRequest {
    let config = get_swap_config();

    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        value: "1000000".into(),
        mode: SwapperMode::ExactIn,
        options: Options {
            slippage: SwapperSlippage {
                mode: SwapperSlippageMode::Auto,
                bps: 50,
            },
            fee: Some(config.referral_fee.clone()),
            preferred_providers: vec![],
            use_max_amount: false,
        },
    }
}
