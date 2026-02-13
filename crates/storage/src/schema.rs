// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "address_type"))]
    pub struct AddressType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "asset_type"))]
    pub struct AssetType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ip_usage_type"))]
    pub struct IpUsageType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "link_type"))]
    pub struct LinkType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "nft_type"))]
    pub struct NftType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_type"))]
    pub struct NotificationType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "platform"))]
    pub struct Platform;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "platform_store"))]
    pub struct PlatformStore;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "redemption_status"))]
    pub struct RedemptionStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "reward_event_type"))]
    pub struct RewardEventType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "reward_redemption_type"))]
    pub struct RewardRedemptionType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "reward_status"))]
    pub struct RewardStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_state"))]
    pub struct TransactionState;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transaction_type"))]
    pub struct TransactionType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "username_status"))]
    pub struct UsernameStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "wallet_source"))]
    pub struct WalletSource;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "wallet_type"))]
    pub struct WalletType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AssetType;

    assets (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 32]
        chain -> Varchar,
        #[max_length = 128]
        token_id -> Nullable<Varchar>,
        asset_type -> AssetType,
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
        is_earnable -> Bool,
        earn_apr -> Nullable<Float8>,
        has_image -> Bool,
    }
}

diesel::table! {
    assets_addresses (id) {
        id -> Int4,
        #[max_length = 32]
        chain -> Varchar,
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 256]
        address -> Varchar,
        #[max_length = 256]
        value -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LinkType;

    assets_links (id) {
        id -> Int4,
        #[max_length = 128]
        asset_id -> Varchar,
        link_type -> LinkType,
        #[max_length = 256]
        url -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    assets_tags (asset_id, tag_id) {
        #[max_length = 128]
        asset_id -> Varchar,
        #[max_length = 64]
        tag_id -> Varchar,
        order -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    assets_usage_ranks (asset_id) {
        #[max_length = 128]
        asset_id -> Varchar,
        usage_rank -> Int4,
        updated_at -> Timestamp,
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
    charts (coin_id, created_at) {
        #[max_length = 255]
        coin_id -> Varchar,
        price -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    charts_daily (coin_id, created_at) {
        #[max_length = 255]
        coin_id -> Varchar,
        price -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    charts_hourly (coin_id, created_at) {
        #[max_length = 255]
        coin_id -> Varchar,
        price -> Float8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    config (key) {
        #[max_length = 64]
        key -> Varchar,
        #[max_length = 256]
        value -> Varchar,
        #[max_length = 256]
        default_value -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Platform;
    use super::sql_types::PlatformStore;

    devices (id) {
        id -> Int4,
        #[max_length = 64]
        device_id -> Varchar,
        is_push_enabled -> Bool,
        platform -> Platform,
        platform_store -> PlatformStore,
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
        #[max_length = 64]
        os -> Varchar,
        #[max_length = 128]
        model -> Varchar,
    }
}

diesel::table! {
    devices_sessions (id) {
        id -> Int4,
        device_id -> Int4,
        created_at -> Timestamp,
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
        is_enabled -> Bool,
        is_enabled_by_provider -> Bool,
        is_buy_enabled -> Bool,
        is_sell_enabled -> Bool,
        unsupported_countries -> Nullable<Jsonb>,
        buy_limits -> Nullable<Jsonb>,
        sell_limits -> Nullable<Jsonb>,
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
        buy_enabled -> Bool,
        sell_enabled -> Bool,
        priority -> Nullable<Int4>,
        priority_threshold_bps -> Nullable<Int4>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_providers_countries (id) {
        #[max_length = 32]
        id -> Varchar,
        #[max_length = 128]
        provider -> Varchar,
        #[max_length = 32]
        alpha2 -> Varchar,
        is_allowed -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_quotes (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 128]
        provider_id -> Varchar,
        #[max_length = 128]
        asset_id -> Varchar,
        fiat_amount -> Float8,
        #[max_length = 32]
        fiat_currency -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_quotes_requests (id) {
        id -> Int4,
        device_id -> Int4,
        #[max_length = 128]
        quote_id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fiat_rates (id) {
        #[max_length = 8]
        id -> Varchar,
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
        country -> Nullable<Varchar>,
        #[max_length = 256]
        provider_transaction_id -> Varchar,
        #[max_length = 256]
        transaction_hash -> Nullable<Varchar>,
        #[max_length = 256]
        address -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        #[max_length = 32]
        transaction_type -> Varchar,
    }
}

diesel::table! {
    fiat_webhooks (id) {
        id -> Int4,
        #[max_length = 32]
        provider -> Varchar,
        #[max_length = 256]
        transaction_id -> Nullable<Varchar>,
        payload -> Jsonb,
        error -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NftType;

    nft_assets (id) {
        #[max_length = 512]
        id -> Varchar,
        #[max_length = 512]
        collection_id -> Varchar,
        #[max_length = 64]
        chain -> Varchar,
        #[max_length = 1024]
        name -> Varchar,
        #[max_length = 4096]
        description -> Varchar,
        #[max_length = 512]
        image_preview_url -> Nullable<Varchar>,
        #[max_length = 64]
        image_preview_mime_type -> Nullable<Varchar>,
        #[max_length = 512]
        resource_url -> Nullable<Varchar>,
        #[max_length = 64]
        resource_mime_type -> Nullable<Varchar>,
        token_type -> NftType,
        #[max_length = 512]
        token_id -> Varchar,
        #[max_length = 512]
        contract_address -> Varchar,
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
        #[max_length = 1024]
        name -> Varchar,
        #[max_length = 4096]
        description -> Varchar,
        #[max_length = 128]
        symbol -> Nullable<Varchar>,
        #[max_length = 128]
        owner -> Nullable<Varchar>,
        #[max_length = 128]
        contract_address -> Varchar,
        #[max_length = 512]
        image_preview_url -> Nullable<Varchar>,
        #[max_length = 64]
        image_preview_mime_type -> Nullable<Varchar>,
        is_verified -> Bool,
        is_enabled -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LinkType;

    nft_collections_links (id) {
        id -> Int4,
        #[max_length = 128]
        collection_id -> Varchar,
        link_type -> LinkType,
        #[max_length = 256]
        url -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nft_reports (id) {
        id -> Int4,
        #[max_length = 512]
        asset_id -> Nullable<Varchar>,
        #[max_length = 512]
        collection_id -> Varchar,
        device_id -> Int4,
        #[max_length = 1024]
        reason -> Nullable<Varchar>,
        reviewed -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NotificationType;

    notifications (id) {
        id -> Int4,
        wallet_id -> Int4,
        asset_id -> Nullable<Varchar>,
        notification_type -> NotificationType,
        is_read -> Bool,
        metadata -> Nullable<Jsonb>,
        read_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    parser_state (chain) {
        chain -> Varchar,
        current_block -> Int8,
        latest_block -> Int8,
        await_blocks -> Int4,
        timeout_between_blocks -> Int4,
        timeout_latest_block -> Int4,
        parallel_blocks -> Int4,
        is_enabled -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        queue_behind_blocks -> Nullable<Int4>,
        block_time -> Int4,
    }
}

diesel::table! {
    perpetuals (id) {
        #[max_length = 128]
        id -> Varchar,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 32]
        provider -> Varchar,
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 128]
        identifier -> Varchar,
        price -> Float8,
        price_percent_change_24h -> Float8,
        open_interest -> Float8,
        volume_24h -> Float8,
        funding -> Float8,
        leverage -> Array<Nullable<Int4>>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    perpetuals_assets (id) {
        id -> Int4,
        #[max_length = 128]
        perpetual_id -> Varchar,
        #[max_length = 256]
        asset_id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    price_alerts (id) {
        id -> Int4,
        #[max_length = 512]
        identifier -> Varchar,
        device_id -> Int4,
        #[max_length = 128]
        asset_id -> Varchar,
        #[max_length = 128]
        currency -> Varchar,
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
        last_updated_at -> Timestamp,
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
    prices_dex (id) {
        #[max_length = 256]
        id -> Varchar,
        #[max_length = 32]
        provider -> Varchar,
        price -> Float8,
        last_updated_at -> Timestamp,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    prices_dex_assets (asset_id, price_feed_id) {
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 256]
        price_feed_id -> Varchar,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    prices_dex_providers (id) {
        #[max_length = 32]
        id -> Varchar,
        enabled -> Bool,
        priority -> Int4,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PlatformStore;

    releases (platform_store) {
        platform_store -> PlatformStore,
        #[max_length = 32]
        version -> Varchar,
        upgrade_required -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
        update_enabled -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RewardStatus;

    rewards (username) {
        #[max_length = 64]
        username -> Varchar,
        status -> RewardStatus,
        #[max_length = 32]
        level -> Nullable<Varchar>,
        points -> Int4,
        #[max_length = 64]
        referrer_username -> Nullable<Varchar>,
        referral_count -> Int4,
        device_id -> Int4,
        is_swap_complete -> Bool,
        #[max_length = 512]
        comment -> Nullable<Varchar>,
        #[max_length = 256]
        disable_reason -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RewardEventType;

    rewards_events (id) {
        id -> Int4,
        #[max_length = 64]
        username -> Varchar,
        event_type -> RewardEventType,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RewardRedemptionType;

    rewards_redemption_options (id) {
        #[max_length = 64]
        id -> Varchar,
        redemption_type -> RewardRedemptionType,
        points -> Int4,
        #[max_length = 128]
        asset_id -> Nullable<Varchar>,
        #[max_length = 64]
        value -> Varchar,
        remaining -> Nullable<Int4>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RedemptionStatus;

    rewards_redemptions (id) {
        id -> Int4,
        #[max_length = 64]
        username -> Varchar,
        #[max_length = 64]
        option_id -> Varchar,
        device_id -> Int4,
        wallet_id -> Int4,
        #[max_length = 512]
        transaction_id -> Nullable<Varchar>,
        status -> RedemptionStatus,
        #[max_length = 1024]
        error -> Nullable<Varchar>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    rewards_referral_attempts (id) {
        id -> Int4,
        #[max_length = 64]
        referrer_username -> Varchar,
        wallet_id -> Int4,
        device_id -> Int4,
        risk_signal_id -> Nullable<Int4>,
        #[max_length = 256]
        reason -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    rewards_referrals (id) {
        id -> Int4,
        #[max_length = 64]
        referrer_username -> Varchar,
        #[max_length = 64]
        referred_username -> Varchar,
        referred_device_id -> Int4,
        risk_signal_id -> Int4,
        verified_at -> Nullable<Timestamp>,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Platform;
    use super::sql_types::PlatformStore;
    use super::sql_types::IpUsageType;

    rewards_risk_signals (id) {
        id -> Int4,
        #[max_length = 64]
        fingerprint -> Varchar,
        #[max_length = 64]
        referrer_username -> Varchar,
        device_id -> Int4,
        device_platform -> Platform,
        device_platform_store -> PlatformStore,
        #[max_length = 32]
        device_os -> Varchar,
        #[max_length = 64]
        device_model -> Varchar,
        #[max_length = 16]
        device_locale -> Varchar,
        #[max_length = 8]
        device_currency -> Varchar,
        #[max_length = 45]
        ip_address -> Varchar,
        #[max_length = 2]
        ip_country_code -> Varchar,
        ip_usage_type -> IpUsageType,
        #[max_length = 128]
        ip_isp -> Varchar,
        ip_abuse_score -> Int4,
        risk_score -> Int4,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AddressType;

    scan_addresses (id) {
        id -> Int4,
        chain -> Varchar,
        #[max_length = 128]
        address -> Varchar,
        #[max_length = 128]
        name -> Nullable<Varchar>,
        #[sql_name = "type"]
        type_ -> AddressType,
        is_verified -> Bool,
        is_fraudulent -> Bool,
        is_memo_required -> Bool,
        updated_at -> Timestamp,
        created_at -> Timestamp,
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
    tags (id) {
        #[max_length = 64]
        id -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TransactionState;
    use super::sql_types::TransactionType;

    transactions (id) {
        id -> Int8,
        #[max_length = 16]
        chain -> Varchar,
        #[max_length = 128]
        hash -> Varchar,
        #[max_length = 256]
        from_address -> Nullable<Varchar>,
        #[max_length = 256]
        to_address -> Nullable<Varchar>,
        #[max_length = 256]
        memo -> Nullable<Varchar>,
        state -> TransactionState,
        kind -> TransactionType,
        #[max_length = 256]
        value -> Nullable<Varchar>,
        asset_id -> Varchar,
        #[max_length = 32]
        fee -> Nullable<Varchar>,
        utxo_inputs -> Nullable<Jsonb>,
        utxo_outputs -> Nullable<Jsonb>,
        fee_asset_id -> Varchar,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions_addresses (id) {
        id -> Int4,
        transaction_id -> Int8,
        #[max_length = 256]
        asset_id -> Varchar,
        #[max_length = 256]
        address -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UsernameStatus;

    usernames (username) {
        #[max_length = 64]
        username -> Varchar,
        wallet_id -> Int4,
        status -> UsernameStatus,
        updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WalletType;
    use super::sql_types::WalletSource;

    wallets (id) {
        id -> Int4,
        #[max_length = 128]
        identifier -> Varchar,
        wallet_type -> WalletType,
        source -> WalletSource,
        created_at -> Timestamp,
    }
}

diesel::table! {
    wallets_addresses (id) {
        id -> Int4,
        #[max_length = 256]
        address -> Varchar,
    }
}

diesel::table! {
    wallets_subscriptions (id) {
        id -> Int4,
        wallet_id -> Int4,
        device_id -> Int4,
        #[max_length = 32]
        chain -> Varchar,
        address_id -> Int4,
        created_at -> Timestamp,
    }
}

diesel::joinable!(assets -> chains (chain));
diesel::joinable!(assets_addresses -> assets (asset_id));
diesel::joinable!(assets_addresses -> chains (chain));
diesel::joinable!(assets_links -> assets (asset_id));
diesel::joinable!(assets_tags -> assets (asset_id));
diesel::joinable!(assets_tags -> tags (tag_id));
diesel::joinable!(assets_usage_ranks -> assets (asset_id));
diesel::joinable!(charts -> prices (coin_id));
diesel::joinable!(charts_daily -> prices (coin_id));
diesel::joinable!(charts_hourly -> prices (coin_id));
diesel::joinable!(devices -> fiat_rates (currency));
diesel::joinable!(devices_sessions -> devices (device_id));
diesel::joinable!(fiat_assets -> assets (asset_id));
diesel::joinable!(fiat_assets -> fiat_providers (provider));
diesel::joinable!(fiat_providers_countries -> fiat_providers (provider));
diesel::joinable!(fiat_quotes -> assets (asset_id));
diesel::joinable!(fiat_quotes -> fiat_providers (provider_id));
diesel::joinable!(fiat_quotes_requests -> devices (device_id));
diesel::joinable!(fiat_quotes_requests -> fiat_quotes (quote_id));
diesel::joinable!(fiat_transactions -> assets (asset_id));
diesel::joinable!(fiat_transactions -> fiat_providers (provider_id));
diesel::joinable!(fiat_webhooks -> fiat_providers (provider));
diesel::joinable!(nft_assets -> chains (chain));
diesel::joinable!(nft_assets -> nft_collections (collection_id));
diesel::joinable!(nft_collections -> chains (chain));
diesel::joinable!(nft_collections_links -> nft_collections (collection_id));
diesel::joinable!(nft_reports -> devices (device_id));
diesel::joinable!(nft_reports -> nft_assets (asset_id));
diesel::joinable!(nft_reports -> nft_collections (collection_id));
diesel::joinable!(notifications -> assets (asset_id));
diesel::joinable!(notifications -> wallets (wallet_id));
diesel::joinable!(parser_state -> chains (chain));
diesel::joinable!(perpetuals -> assets (asset_id));
diesel::joinable!(perpetuals_assets -> assets (asset_id));
diesel::joinable!(perpetuals_assets -> perpetuals (perpetual_id));
diesel::joinable!(price_alerts -> assets (asset_id));
diesel::joinable!(price_alerts -> devices (device_id));
diesel::joinable!(price_alerts -> fiat_rates (currency));
diesel::joinable!(prices_assets -> assets (asset_id));
diesel::joinable!(prices_assets -> prices (price_id));
diesel::joinable!(prices_dex -> prices_dex_providers (provider));
diesel::joinable!(prices_dex_assets -> assets (asset_id));
diesel::joinable!(prices_dex_assets -> prices_dex (price_feed_id));
diesel::joinable!(rewards -> devices (device_id));
diesel::joinable!(rewards -> usernames (username));
diesel::joinable!(rewards_events -> usernames (username));
diesel::joinable!(rewards_redemption_options -> assets (asset_id));
diesel::joinable!(rewards_redemptions -> devices (device_id));
diesel::joinable!(rewards_redemptions -> rewards (username));
diesel::joinable!(rewards_redemptions -> rewards_redemption_options (option_id));
diesel::joinable!(rewards_redemptions -> wallets (wallet_id));
diesel::joinable!(rewards_referral_attempts -> devices (device_id));
diesel::joinable!(rewards_referral_attempts -> rewards (referrer_username));
diesel::joinable!(rewards_referral_attempts -> rewards_risk_signals (risk_signal_id));
diesel::joinable!(rewards_referral_attempts -> wallets (wallet_id));
diesel::joinable!(rewards_referrals -> devices (referred_device_id));
diesel::joinable!(rewards_referrals -> rewards_risk_signals (risk_signal_id));
diesel::joinable!(rewards_risk_signals -> devices (device_id));
diesel::joinable!(rewards_risk_signals -> rewards (referrer_username));
diesel::joinable!(scan_addresses -> chains (chain));
diesel::joinable!(subscriptions_addresses_exclude -> chains (chain));
diesel::joinable!(transactions -> chains (chain));
diesel::joinable!(transactions_addresses -> assets (asset_id));
diesel::joinable!(transactions_addresses -> transactions (transaction_id));
diesel::joinable!(usernames -> wallets (wallet_id));
diesel::joinable!(wallets_subscriptions -> chains (chain));
diesel::joinable!(wallets_subscriptions -> devices (device_id));
diesel::joinable!(wallets_subscriptions -> wallets (wallet_id));
diesel::joinable!(wallets_subscriptions -> wallets_addresses (address_id));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    assets_addresses,
    assets_links,
    assets_tags,
    assets_usage_ranks,
    chains,
    charts,
    charts_daily,
    charts_hourly,
    config,
    devices,
    devices_sessions,
    fiat_assets,
    fiat_providers,
    fiat_providers_countries,
    fiat_quotes,
    fiat_quotes_requests,
    fiat_rates,
    fiat_transactions,
    fiat_webhooks,
    nft_assets,
    nft_collections,
    nft_collections_links,
    nft_reports,
    notifications,
    parser_state,
    perpetuals,
    perpetuals_assets,
    price_alerts,
    prices,
    prices_assets,
    prices_dex,
    prices_dex_assets,
    prices_dex_providers,
    releases,
    rewards,
    rewards_events,
    rewards_redemption_options,
    rewards_redemptions,
    rewards_referral_attempts,
    rewards_referrals,
    rewards_risk_signals,
    scan_addresses,
    subscriptions_addresses_exclude,
    tags,
    transactions,
    transactions_addresses,
    usernames,
    wallets,
    wallets_addresses,
    wallets_subscriptions,
);
