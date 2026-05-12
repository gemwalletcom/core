use crate::{
    ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapperProvider, SwapperQuoteAsset,
    cetus::{
        constants::CETUS,
        model::{FlattenedPath, Path, RouterData},
    },
    fees::ReferralFee,
    models::Options,
};
use primitives::{AssetId, Chain};
use sui_types::Address;

pub(crate) fn quote(slippage_bps: u32) -> Quote {
    Quote {
        from_value: "1000".to_string(),
        to_value: "2000".to_string(),
        data: ProviderData {
            provider: ProviderType::new(SwapperProvider::CetusAggregator),
            routes: vec![Route {
                input: AssetId::from_chain(Chain::Sui),
                output: AssetId::from_chain(Chain::Sui),
                route_data: String::new(),
            }],
            slippage_bps,
        },
        request: QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Sui)),
            wallet_address: Address::ZERO.to_string(),
            destination_address: Address::ZERO.to_string(),
            value: "1000".to_string(),
            options: Options::default(),
        },
        eta_in_seconds: Some(0),
    }
}

pub(crate) fn router(amount_out: u64) -> RouterData {
    RouterData {
        request_id: "quote".to_string(),
        amount_out,
        paths: vec![],
        packages: None,
    }
}

pub(crate) fn route_path(direction: bool, published_at: Option<String>) -> Path {
    Path {
        id: "0x1".to_string(),
        direction,
        provider: CETUS.to_string(),
        from: "0x2::sui::SUI".to_string(),
        target: "0xabc::coin::A".to_string(),
        amount_in: 123,
        published_at,
        extended_details: None,
    }
}

pub(crate) fn flattened_path(path: Path, is_last_use_of_intermediate_token: bool) -> FlattenedPath {
    FlattenedPath {
        path,
        is_last_use_of_intermediate_token,
    }
}

pub(crate) fn referral_fee(bps: u32) -> ReferralFee {
    ReferralFee {
        address: Address::ZERO.to_string(),
        bps,
    }
}
