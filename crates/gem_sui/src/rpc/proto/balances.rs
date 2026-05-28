use gem_encoding::protobuf::{proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 state_service.proto:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/state_service.proto

#[derive(Clone, Debug, Default)]
pub struct GetBalanceRequest {
    pub owner: Option<String>,
    pub coin_type: Option<String>,
}

proto_encode!(GetBalanceRequest {
    1 => owner: optional_string,
    2 => coin_type: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct GetBalanceResponse {
    pub balance: Option<Balance>,
}

proto_decode!(GetBalanceResponse {
    1 => balance: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ListBalancesRequest {
    pub owner: Option<String>,
    pub page_size: Option<u32>,
    pub page_token: Option<Vec<u8>>,
}

proto_encode!(ListBalancesRequest {
    1 => owner: optional_string,
    2 => page_size: optional_varint_u32,
    3 => page_token: optional_bytes,
});

#[derive(Clone, Debug, Default)]
pub struct ListBalancesResponse {
    pub balances: Vec<Balance>,
    pub next_page_token: Option<Vec<u8>>,
}

proto_decode!(ListBalancesResponse {
    1 => balances: repeated_message,
    2 => next_page_token: optional_bytes,
});

#[derive(Clone, Debug, Default)]
pub struct Balance {
    pub coin_type: Option<String>,
    pub balance: Option<u64>,
    pub address_balance: Option<u64>,
}

proto_decode!(Balance {
    1 => coin_type: optional_string,
    3 => balance: optional_varint_u64,
    4 => address_balance: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub struct GetCoinInfoRequest {
    pub coin_type: Option<String>,
}

proto_encode!(GetCoinInfoRequest {
    1 => coin_type: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct GetCoinInfoResponse {
    pub metadata: Option<CoinMetadata>,
}

proto_decode!(GetCoinInfoResponse {
    2 => metadata: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct CoinMetadata {
    pub decimals: Option<u32>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

proto_decode!(CoinMetadata {
    2 => decimals: optional_varint_u32,
    3 => name: optional_string,
    4 => symbol: optional_string,
});
