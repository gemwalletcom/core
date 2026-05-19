use gem_ton::address::Address;
use gem_ton::models::TokenInfo;
use primitives::Address as _;

// TODO: replace this hardcoded allowlist with a proper spam filter — e.g. a DB-backed verified
// collections table populated from an authoritative source (Getgems / Fragment / TonScan),
// or a heuristic based on collection age / holder count / on-chain verification signals.
const VERIFIED_MARKETPLACES: &[&str] = &["getgems.io"];
const VERIFIED_COLLECTIONS: &[&str] = &[
    "EQCA14o1-VWhS2efqoh_9M1b_A9DtKTuoqfmkn83AbJzwnPi", // Telegram Usernames
    "EQC3dNlesgVD8YbAazcauIrXBPfiVhMMr5YYk2in0Mtsz0Bz", // TON DNS (.ton domains)
    "EQAOQdwdw8kGftJCSFgOErM1mBjYPe4DBPq8-AhF6vr9si5N", // Anonymous Telegram Numbers
];

pub fn is_verified(address: &Address, info: &TokenInfo) -> bool {
    is_verified_collection(address) || is_verified_marketplace(info.extra.as_ref().and_then(|extra| extra.marketplace.as_deref()))
}

fn is_verified_collection(address: &Address) -> bool {
    VERIFIED_COLLECTIONS.contains(&address.encode().as_str())
}

fn is_verified_marketplace(marketplace: Option<&str>) -> bool {
    match marketplace {
        Some(marketplace) => VERIFIED_MARKETPLACES.contains(&marketplace),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_ton::models::TokenInfoExtra;

    #[test]
    fn test_is_verified() {
        let address = Address::try_parse_hex("0:80D78A35F955A14B679FAA887FF4CD5BFC0F43B4A4EEA2A7E6927F3701B273C2").unwrap();
        let info = mock_token_info(None);
        assert!(is_verified(&address, &info));

        let other = Address::try_parse_hex("0:0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert!(is_verified(&other, &mock_token_info(Some("getgems.io"))));
        assert!(!is_verified(&other, &mock_token_info(Some("other.io"))));
    }

    fn mock_token_info(marketplace: Option<&str>) -> TokenInfo {
        TokenInfo {
            valid: true,
            name: Some("Collection".to_string()),
            description: None,
            image: None,
            extra: Some(TokenInfoExtra {
                domain: None,
                marketplace: marketplace.map(str::to_string),
            }),
        }
    }
}
