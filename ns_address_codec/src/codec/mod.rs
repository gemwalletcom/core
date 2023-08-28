pub trait Codec {
    fn encode(bytes: Vec<u8>) -> String;
    fn decode(string: &str) -> Vec<u8>;
}
