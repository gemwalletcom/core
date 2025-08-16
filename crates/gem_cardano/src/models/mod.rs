pub mod account;
pub mod block;
pub mod transaction;
pub mod utxo;

pub type UInt64 = u64;

pub use account::*;
pub use block::*;
pub use transaction::*;
pub use utxo::*;
