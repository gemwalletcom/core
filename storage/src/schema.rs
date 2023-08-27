// @generated automatically by Diesel CLI.

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
        created_at -> Nullable<Timestamp>,
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
    tokenlists (id) {
        id -> Int4,
        chain -> Varchar,
        url -> Varchar,
        version -> Int4,
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

diesel::allow_tables_to_appear_in_same_query!(
    charts,
    fiat_assets,
    fiat_rates,
    nodes,
    prices,
    tokenlists,
    versions,
);
