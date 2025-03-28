// https://github.com/pyth-network/pyth-sdk-rs/blob/8c56e4f6a3efee94da9cafa614aba94c29c19376/pyth-sdk-solana/src/state.rs#L288
/// Represents a price update (latest price, confidence, status)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Price {
    pub price: f64,
}
