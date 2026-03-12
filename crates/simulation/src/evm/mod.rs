mod approval_method;
mod approval_request;
mod approval_value;
mod decode;

#[cfg(feature = "rpc")]
mod client;

pub use decode::{simulate_eip712_message, simulate_evm_calldata};

#[cfg(feature = "rpc")]
pub use client::SimulationClient;

#[cfg(test)]
mod tests;
