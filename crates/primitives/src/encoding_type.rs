use typeshare::typeshare;

#[derive(Clone, Debug, PartialEq)]
#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
pub enum EncodingType {
    Hex,
    Base58,
    Base32,
}
