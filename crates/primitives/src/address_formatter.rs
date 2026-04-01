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

    pub fn short(address: &str, chain: Option<Chain>) -> String {
        Self::format(address, chain, AddressFormatStyle::Short)
    }

    pub fn full(address: &str) -> String {
        Self::format(address, None, AddressFormatStyle::Full)
    }

    pub fn extra(address: &str, chain: Option<Chain>, extra: usize) -> String {
        Self::format(address, chain, AddressFormatStyle::Extra { extra: extra as u32 })
    }

    fn truncate(address: &str, chain: Option<Chain>, extra: usize) -> String {
        let leading = Self::leading_chars(chain) + extra;
        let trailing = Self::EDGE_CHARS + extra;
        let char_count = address.chars().count();

        if char_count <= leading + trailing {
            return address.to_string();
        }

        let middle = address.chars().skip(leading).take(char_count - leading - trailing).collect::<String>();

        address.replace(&middle, Self::CONNECTOR)
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
            "0x123123...21321312"
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
    }
}
