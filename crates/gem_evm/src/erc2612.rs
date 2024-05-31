#[derive(Debug, PartialEq)]
pub struct Permit {
    pub value: String,
    pub deadline: u64,
    pub v: u8,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}
