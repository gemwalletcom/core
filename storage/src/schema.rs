// @generated automatically by Diesel CLI.

diesel::table! {
    assets (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 16]
        chain -> Varchar,
        #[max_length = 128]
        token_id -> Nullable<Varchar>,
        #[max_length = 16]
        asset_type -> Varchar,
        #[max_length = 64]
        name -> Varchar,
        #[max_length = 16]
        symbol -> Varchar,
        decimals -> Int4,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    assets_details (asset_id) {
        #[max_length = 128]
        asset_id -> Varchar,
        #[max_length = 64]
        homepage -> Nullable<Varchar>,
        #[max_length = 64]
        explorer -> Nullable<Varchar>,
        #[max_length = 64]
        twitter -> Nullable<Varchar>,
        #[max_length = 64]
        telegram -> Nullable<Varchar>,
        #[max_length = 64]
        github -> Nullable<Varchar>,
        #[max_length = 64]
        youtube -> Nullable<Varchar>,
        #[max_length = 64]
        facebook -> Nullable<Varchar>,
        #[max_length = 64]
        reddit -> Nullable<Varchar>,
        #[max_length = 64]
        coingecko -> Nullable<Varchar>,
        #[max_length = 64]
        coinmarketcap -> Nullable<Varchar>,
    }
}

diesel::table! {
    charts (id) {
        id -> Int4,
        coin_id -> Varchar,
        date -> Timestamp,
        price -> Float8,
        market_cap -> Float8,
        volume -> Float8,
    }
}

diesel::table! {
    devices (id) {
        id -> Int4,
        #[max_length = 32]
        device_id -> Varchar,
        is_push_enabled -> Bool,
        #[max_length = 8]
        platform -> Varchar,
        #[max_length = 256]
        token -> Varchar,
        #[max_length = 8]
        locale -> Varchar,
        #[max_length = 8]
        version -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        #[max_length = 8]
        currency -> Varchar,
        subscriptions_version -> Int4,
    }
}

diesel::table! {
    fiat_assets (id) {
        id -> Int4,
        asset -> Varchar,
        provider -> Varchar,
        symbol -> Varchar,
        network -> Nullable<Varchar>,
    }
}

diesel::table! {
    fiat_rates (id) {
        id -> Int4,
        symbol -> Varchar,
        name -> Varchar,
        rate -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nodes (id) {
        id -> Int4,
        chain -> Varchar,
        url -> Varchar,
        status -> Varchar,
        priority -> Int4,
        updated_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    parser_state (chain) {
        chain -> Varchar,
        current_block -> Int4,
        latest_block -> Int4,
        await_blocks -> Int4,
        timeout_between_blocks -> Int4,
        parallel_blocks -> Int4,
        is_enabled -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        block_created_at -> Timestamp,
    }
}

diesel::table! {
    prices (asset_id) {
        asset_id -> Varchar,
        coin_id -> Varchar,
        price -> Float8,
        price_change_percentage_24h -> Float8,
        market_cap -> Float8,
        market_cap_rank -> Int4,
        total_volume -> Float8,
        last_updated_at -> Timestamp,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Int4,
        device_id -> Int4,
        chain -> Varchar,
        #[max_length = 256]
        address -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        wallet_index -> Int4,
    }
}

diesel::table! {
    tokenlists (id) {
        id -> Int4,
        chain -> Varchar,
        url -> Varchar,
        version -> Int4,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        chain -> Varchar,
        #[max_length = 256]
        hash -> Varchar,
        #[max_length = 256]
        from_address -> Nullable<Varchar>,
        #[max_length = 256]
        to_address -> Nullable<Varchar>,
        #[max_length = 256]
        contract -> Nullable<Varchar>,
        #[max_length = 256]
        memo -> Nullable<Varchar>,
        sequence -> Nullable<Int4>,
        block_number -> Int4,
        #[max_length = 16]
        kind -> Varchar,
        #[max_length = 32]
        value -> Nullable<Varchar>,
        asset_id -> Nullable<Varchar>,
        #[max_length = 32]
        fee -> Nullable<Varchar>,
        fee_asset_id -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        #[max_length = 16]
        state -> Varchar,
    }
}

diesel::table! {
    versions (id) {
        id -> Int4,
        platform -> Varchar,
        production -> Varchar,
        beta -> Varchar,
        alpha -> Varchar,
    }
}

diesel::joinable!(assets_details -> assets (asset_id));
diesel::joinable!(subscriptions -> devices (device_id));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    assets_details,
    charts,
    devices,
    fiat_assets,
    fiat_rates,
    nodes,
    parser_state,
    prices,
    subscriptions,
    tokenlists,
    transactions,
    versions,
);
