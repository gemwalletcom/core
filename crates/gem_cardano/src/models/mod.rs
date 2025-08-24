pub mod account;
pub mod block;
pub mod rpc;
pub mod transaction;
pub mod utxo;

pub type UInt64 = u64;

pub use account::*;
pub use block::*;
pub use rpc::*;
pub use transaction::*;
pub use utxo::*;
