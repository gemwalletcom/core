// TODO: replace this hardcoded allowlist with a proper spam filter — e.g. a DB-backed verified
// collections table populated from an authoritative source (Getgems / Fragment / TonScan),
// or a heuristic based on collection age / holder count / on-chain verification signals.
const VERIFIED_COLLECTIONS: &[&str] = &[
    "0:80D78A35F955A14B679FAA887FF4CD5BFC0F43B4A4EEA2A7E6927F3701B273C2", // Telegram Usernames
    "0:B774D95EB20543F186C06B371AB88AD704F7E256130CAF96189368A7D0CB6CCF", // TON DNS (.ton domains)
    "0:0E41DC1DC3C9067ED24248580E12B3359818D83DEE0304FABCF80845EAFAFDB2", // Anonymous Telegram Numbers
];

pub fn is_verified(address: &str) -> bool {
    VERIFIED_COLLECTIONS.contains(&address)
}
