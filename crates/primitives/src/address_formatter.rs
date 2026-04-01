use crate::{Chain, ChainType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFormatStyle {
    Short,
    Full,
    Extra { extra: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressFormatter;

impl AddressFormatter {
    const CONNECTOR: &str = "...";
    const EDGE_CHARS: usize = 5;

    pub fn format(address: &str, chain: Option<Chain>, style: AddressFormatStyle) -> String {
        match style {
            AddressFormatStyle::Short => Self::truncate(address, chain, 0),
            AddressFormatStyle::Full => address.to_string(),
            AddressFormatStyle::Extra { extra } => Self::truncate(address, chain, extra as usize),
        }
    }

    fn truncate(address: &str, chain: Option<Chain>, extra: usize) -> String {
        let leading = Self::leading_chars(chain) + extra;
        let trailing = Self::EDGE_CHARS + extra;
        let chars = address.chars().collect::<Vec<_>>();
        let char_count = chars.len();

        if char_count <= leading + trailing {
            return address.to_string();
        }

        let start = chars.iter().take(leading).copied().collect::<String>();
        let end = chars.iter().skip(char_count - trailing).copied().collect::<String>();
        format!("{start}{}{end}", Self::CONNECTOR)
    }

    fn leading_chars(chain: Option<Chain>) -> usize {
        match chain.map(|chain| chain.chain_type()) {
            Some(ChainType::Ethereum) => Self::EDGE_CHARS + "0x".len(),
            Some(ChainType::Bitcoin | ChainType::Aptos) => Self::EDGE_CHARS + 1,
            Some(_) | None => Self::EDGE_CHARS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_formatter() {
        assert_eq!(
            AddressFormatter::format("0x12312321321312", Some(Chain::Ethereum), AddressFormatStyle::Short),
            "0x12312...21312"
        );
        assert_eq!(
            AddressFormatter::format("0x12312321321312", Some(Chain::Aptos), AddressFormatStyle::Short),
            "0x1231...21312"
        );
        assert_eq!(
            AddressFormatter::format("GLNvG5Ly4cK512oQeJqnwLftwfoPZ4skyDwZWzxorYQ9", Some(Chain::Solana), AddressFormatStyle::Short),
            "GLNvG...orYQ9"
        );
        assert_eq!(
            AddressFormatter::format(
                "bc1qx2x5cqhymfcnjtg902ky6u5t5htmt7fvqztdsm028hkrvxcl4t2sjtpd9l",
                Some(Chain::Bitcoin),
                AddressFormatStyle::Short
            ),
            "bc1qx2...tpd9l"
        );
        assert_eq!(
            AddressFormatter::format("0x1231232221321312", Some(Chain::Ethereum), AddressFormatStyle::Extra { extra: 2 }),
            "0x1231232...1321312"
        );
        assert_eq!(
            AddressFormatter::format("0x12313332321321312", Some(Chain::Aptos), AddressFormatStyle::Extra { extra: 2 }),
            "0x123133...1321312"
        );
        assert_eq!(
            AddressFormatter::format("0x1231232221321312", Some(Chain::Ethereum), AddressFormatStyle::Full),
            "0x1231232221321312"
        );
        assert_eq!(AddressFormatter::format("bc1short", Some(Chain::Bitcoin), AddressFormatStyle::Short), "bc1short");
        assert_eq!(
            AddressFormatter::format("abcXXmiddleXXcba", Some(Chain::Solana), AddressFormatStyle::Short),
            "abcXX...XXcba"
        );
    }
}
