// @generated automatically by Diesel CLI.

diesel::table! {
    assets (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 32]
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
        rank -> Int4,
        is_enabled -> Bool,
        is_buyable -> Bool,
        is_sellable -> Bool,
        is_swappable -> Bool,
        is_stakeable -> Bool,
        staking_apr -> Nullable<Float8>,
    }
}

diesel::table! {
    assets_links (id) {
        id -> Int4,
        #[max_length = 128]
        asset_id -> Varchar,
        #[max_length = 32]
        link_type -> Varchar,
        #[max_length = 256]
        url -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    assets_types (id) {
        #[max_length = 32]
        id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    chains (id) {
        #[max_length = 32]
        id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
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
        #[max_length = 32]
        platform_store -> Nullable<Varchar>,
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
        is_price_alerts_enabled -> Bool,
    }
}

diesel::table! {
    fiat_assets (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 128]
        asset_id -> Nullable<Varchar>,
        #[max_length = 128]
        provider -> Varchar,
        #[max_length = 128]
        code -> Varchar,
        #[max_length = 128]
        symbol -> Varchar,
        #[max_length = 128]
        network -> Nullable<Varchar>,
        #[max_length = 128]
        token_id -> Nullable<Varchar>,
        enabled -> Bool,
        enabled_by_provider -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_providers (id) {
        #[max_length = 32]
        id -> Varchar,
        #[max_length = 32]
        name -> Varchar,
        enabled -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_rates (id) {
        id -> Int4,
        symbol -> Varchar,
        name -> Varchar,
        rate -> Float8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    fiat_transactions (id) {
        id -> Int4,
        #[max_length = 128]
        provider_id -> Varchar,
        #[max_length = 128]
        asset_id -> Nullable<Varchar>,
        #[max_length = 32]
        symbol -> Varchar,
        fiat_amount -> Float8,
        #[max_length = 32]
        fiat_currency -> Varchar,
        #[max_length = 32]
        status -> Varchar,
        #[max_length = 256]
        provider_transaction_id -> Varchar,
        #[max_length = 256]
        transaction_hash -> Nullable<Varchar>,
        #[max_length = 256]
        address -> Nullable<Varchar>,
        fee_provider -> Nullable<Float8>,
        fee_network -> Nullable<Float8>,
        fee_partner -> Nullable<Float8>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        #[max_length = 32]
        transaction_type -> Varchar,
    }
}

diesel::table! {
    link_types (id) {
        #[max_length = 32]
        id -> Varchar,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    nft_assets (id) {
        #[max_length = 512]
        id -> Varchar,
        #[max_length = 64]
        collection_id -> Varchar,
        #[max_length = 64]
        chain -> Varchar,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 1024]
        description -> Varchar,
        #[max_length = 512]
        image_url -> Varchar,
        #[max_length = 32]
        token_type -> Varchar,
        #[max_length = 512]
        token_id -> Varchar,
        attributes -> Jsonb,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nft_collections (id) {
        #[max_length = 512]
        id -> Varchar,
        #[max_length = 64]
        chain -> Varchar,
        #[max_length = 256]
        name -> Varchar,
        #[max_length = 1024]
        description -> Varchar,
        #[max_length = 128]
        symbol -> Nullable<Varchar>,
        #[max_length = 256]
        url -> Nullable<Varchar>,
        #[max_length = 128]
        owner -> Nullable<Varchar>,
        #[max_length = 128]
        contrtact_address -> Varchar,
        #[max_length = 512]
        image_url -> Nullable<Varchar>,
        #[max_length = 128]
        project_url -> Nullable<Varchar>,
        #[max_length = 128]
        opensea_url -> Nullable<Varchar>,
        #[max_length = 128]
        project_x_username -> Nullable<Varchar>,
        is_verified -> Bool,
        is_enable -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nft_links (id) {
        id -> Int4,
        #[max_length = 128]
        collection_id -> Varchar,
        #[max_length = 32]
        link_type -> Varchar,
        #[max_length = 256]
        url -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nft_types (id) {
        #[max_length = 32]
        id -> Varchar,
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
    }
}

diesel::table! {
    price_alerts (id) {
        id -> Int4,
        device_id -> Int4,
        #[max_length = 128]
        asset_id -> Varchar,
        #[max_length = 16]
        price_direction -> Nullable<Varchar>,
        price -> Nullable<Float8>,
        price_percent_change -> Nullable<Float8>,
        last_notified_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    prices (id) {
        #[max_length = 256]
        id -> Varchar,
        price -> Float8,
        price_change_percentage_24h -> Float8,
        market_cap -> Float8,
        market_cap_fdv -> Float8,
        market_cap_rank -> Int4,
        total_volume -> Float8,
        circulating_supply -> Float8,
        total_supply -> Float8,
        max_supply -> Float8,
        last_updated_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        all_time_high_date -> Nullable<Timestamp>,
        all_time_low_date -> Nullable<Timestamp>,
        all_time_high -> Float8,
        all_time_low -> Float8,
    }
}

diesel::table! {
    prices_assets (asset_id) {
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 256]
        price_id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    releases (platform_store) {
        #[max_length = 32]
        platform_store -> Varchar,
        #[max_length = 32]
        version -> Varchar,
        upgrade_required -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    scan_addresses (id) {
        id -> Int4,
        chain -> Varchar,
        #[max_length = 128]
        address -> Varchar,
        #[max_length = 64]
        name -> Nullable<Varchar>,
        #[sql_name = "type"]
        #[max_length = 32]
        type_ -> Nullable<Varchar>,
        is_verified -> Bool,
        is_fraudulent -> Bool,
        is_memo_required -> Bool,
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
    subscriptions_addresses_exclude (address) {
        #[max_length = 128]
        address -> Varchar,
        #[max_length = 32]
        chain -> Varchar,
        #[max_length = 64]
        name -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
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
        #[max_length = 256]
        id -> Varchar,
        #[max_length = 16]
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
        state -> Varchar,
        #[max_length = 16]
        kind -> Varchar,
        #[max_length = 256]
        value -> Nullable<Varchar>,
        asset_id -> Varchar,
        #[max_length = 32]
        fee -> Nullable<Varchar>,
        utxo_inputs -> Nullable<Jsonb>,
        utxo_outputs -> Nullable<Jsonb>,
        fee_asset_id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    transactions_addresses (id) {
        id -> Int4,
        #[max_length = 32]
        chain_id -> Varchar,
        #[max_length = 256]
        transaction_id -> Varchar,
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 256]
        address -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::joinable!(assets -> assets_types (asset_type));
diesel::joinable!(assets -> chains (chain));
diesel::joinable!(assets_links -> assets (asset_id));
diesel::joinable!(assets_links -> link_types (link_type));
diesel::joinable!(fiat_assets -> assets (asset_id));
diesel::joinable!(fiat_assets -> fiat_providers (provider));
diesel::joinable!(fiat_transactions -> assets (asset_id));
diesel::joinable!(fiat_transactions -> fiat_providers (provider_id));
diesel::joinable!(nft_assets -> chains (chain));
diesel::joinable!(nft_assets -> nft_collections (collection_id));
diesel::joinable!(nft_assets -> nft_types (token_type));
diesel::joinable!(nft_collections -> chains (chain));
diesel::joinable!(nft_links -> link_types (link_type));
diesel::joinable!(nft_links -> nft_collections (collection_id));
diesel::joinable!(nodes -> chains (chain));
diesel::joinable!(parser_state -> chains (chain));
diesel::joinable!(price_alerts -> assets (asset_id));
diesel::joinable!(price_alerts -> devices (device_id));
diesel::joinable!(prices_assets -> assets (asset_id));
diesel::joinable!(prices_assets -> prices (price_id));
diesel::joinable!(scan_addresses -> chains (chain));
diesel::joinable!(subscriptions -> chains (chain));
diesel::joinable!(subscriptions -> devices (device_id));
diesel::joinable!(subscriptions_addresses_exclude -> chains (chain));
diesel::joinable!(tokenlists -> chains (chain));
diesel::joinable!(transactions -> chains (chain));
diesel::joinable!(transactions_addresses -> assets (asset_id));
diesel::joinable!(transactions_addresses -> chains (chain_id));
diesel::joinable!(transactions_addresses -> transactions (transaction_id));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    assets_links,
    assets_types,
    chains,
    devices,
    fiat_assets,
    fiat_providers,
    fiat_rates,
    fiat_transactions,
    link_types,
    nft_assets,
    nft_collections,
    nft_links,
    nft_types,
    nodes,
    parser_state,
    price_alerts,
    prices,
    prices_assets,
    releases,
    scan_addresses,
    subscriptions,
    subscriptions_addresses_exclude,
    tokenlists,
    transactions,
    transactions_addresses,
);
