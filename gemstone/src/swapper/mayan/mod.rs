pub mod forwarder;
pub mod provider;
pub mod relayer;
pub mod swift;

pub const MAYAN_PROGRAM_ID: &str = "FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf";
pub const MAYAN_FORWARDER_CONTRACT: &str = "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2";

pub use provider::MayanSwiftProvider;
