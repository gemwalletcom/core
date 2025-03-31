pub mod forwarder;
pub mod models;
pub mod provider;
pub mod relayer;
pub mod swift;
pub(crate) mod tx_builder;

pub const MAYAN_PROGRAM_ID: &str = "FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf";
pub const MAYAN_FORWARDER_CONTRACT: &str = "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2";
pub const FORWARD_ERC20_GAS_LIMIT: u64 = 190_000;
pub const FORWARD_SWAP_ERC20_GAS_LIMIT: u64 = 500_000;

pub use models::{Quote, QuoteOptions, QuoteResponse, QuoteType, QuoteUrlParams, Token};
pub use provider::MayanSwiftProvider;
