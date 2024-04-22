use std::error::Error;
pub trait Codec {
    fn encode(bytes: Vec<u8>) -> Result<String, Box<dyn Error + Send + Sync>>;
    fn decode(string: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>>;
}
