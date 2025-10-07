use crate::{
    SwapperSlippage, SwapperSlippageMode,
    config::{ReferralFee, ReferralFees},
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
            fee: Some(ReferralFees {
                evm: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                evm_bridge: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                solana: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                thorchain: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                sui: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                ton: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
                tron: ReferralFee {
                    address: "g1".into(),
                    bps: 100,
                },
            }),
            preferred_providers: vec![],
        },
    }
}
