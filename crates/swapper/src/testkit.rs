use crate::{
    SwapperSlippage, SwapperSlippageMode,
    swap_config::{SwapReferralFee, SwapReferralFees},
};

use super::{SwapperMode, SwapperOptions, SwapperQuoteRequest};
use primitives::AssetId;

pub fn mock_quote(from_asset: AssetId, to_asset: AssetId) -> SwapperQuoteRequest {
    SwapperQuoteRequest {
        from_asset: from_asset.into(),
        to_asset: to_asset.into(),
        wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
        value: "1000000".into(),
        mode: SwapperMode::ExactIn,
        options: SwapperOptions {
            slippage: SwapperSlippage {
                mode: SwapperSlippageMode::Auto,
                bps: 100,
            },
            fee: Some(SwapReferralFees {
                evm: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                evm_bridge: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                solana: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                thorchain: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                sui: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                ton: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                tron: SwapReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
            }),
            preferred_providers: vec![],
        },
    }
}
