use std::error::Error;

pub type MessageResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub trait MessageEncode {
    fn encode(&self) -> Vec<u8>;
}

pub trait MessageDecode: Sized {
    fn decode(data: &[u8]) -> MessageResult<Self>;
}

pub trait Message: MessageEncode + MessageDecode {}

impl<T> Message for T where T: MessageEncode + MessageDecode {}
