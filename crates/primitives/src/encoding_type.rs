use typeshare::typeshare;

#[derive(Clone, Debug, PartialEq)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum EncodingType {
    Hex,
    Base58,
}
