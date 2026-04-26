use gem_ton::address::Address;

// TODO: replace this hardcoded allowlist with a proper spam filter — e.g. a DB-backed verified
// collections table populated from an authoritative source (Getgems / Fragment / TonScan),
// or a heuristic based on collection age / holder count / on-chain verification signals.
const VERIFIED_COLLECTIONS: &[&str] = &[
    "EQCA14o1-VWhS2efqoh_9M1b_A9DtKTuoqfmkn83AbJzwnPi", // Telegram Usernames
    "EQC3dNlesgVD8YbAazcauIrXBPfiVhMMr5YYk2in0Mtsz0Bz", // TON DNS (.ton domains)
    "EQAOQdwdw8kGftJCSFgOErM1mBjYPe4DBPq8-AhF6vr9si5N", // Anonymous Telegram Numbers
];

pub fn is_verified(address: &Address) -> bool {
    VERIFIED_COLLECTIONS.contains(&address.to_base64_url().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_verified() {
        let address = Address::from_hex_str("0:80D78A35F955A14B679FAA887FF4CD5BFC0F43B4A4EEA2A7E6927F3701B273C2").unwrap();
        assert!(is_verified(&address));

        let other = Address::from_hex_str("0:0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert!(!is_verified(&other));
    }
}
