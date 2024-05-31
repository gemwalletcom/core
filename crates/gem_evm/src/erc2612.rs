#[derive(Debug, PartialEq)]
pub struct Permit {
    pub value: String,
    pub deadline: String,
    pub v: u8,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}
