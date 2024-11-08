use crate::chain::Chain;

pub struct AddressFormatter {}

impl AddressFormatter {
    pub fn short(_chain: Chain, address: &str) -> String {
        let len = address.len();
        if len < 10 {
            return address.to_string();
        }
        let first_four = if address.starts_with("0x") { &address[..6] } else { &address[..4] };
        let last_four = &address[len - 4..];
        format!("{}...{}", first_four, last_four)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_address() {
        assert_eq!(AddressFormatter::short(Chain::Bitcoin, "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"), "1BvB...NVN2");
        assert_eq!(
            AddressFormatter::short(Chain::Ethereum, "0x1CeDC0f3Af8f9841B0a1F5c1a4DDc6e1a1629074"),
            "0x1CeD...9074"
        );
    }
}
