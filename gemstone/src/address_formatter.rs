use primitives::{AddressFormatStyle, AddressFormatter, Chain};

pub type GemAddressFormatStyle = AddressFormatStyle;

#[uniffi::remote(Enum)]
pub enum GemAddressFormatStyle {
    Short,
    Full,
    Extra { extra: u32 },
}

#[uniffi::export]
pub fn format_address(address: &str, chain: Option<Chain>, style: GemAddressFormatStyle) -> String {
    AddressFormatter::format(address, chain, style)
}
