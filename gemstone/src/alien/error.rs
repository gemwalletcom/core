pub type AlienError = swapper::AlienError;

#[uniffi::remote(Enum)]
pub enum AlienError {
    Network(String),
    Timeout,
    Http { status: u16, body: Vec<u8> },
    Serialization(String),
}
