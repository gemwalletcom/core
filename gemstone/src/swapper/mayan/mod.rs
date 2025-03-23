pub mod forwarder;
pub mod provider;
pub mod relayer;
pub mod swift;

pub const MAYAN_PROGRAM_ID: &str = "FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf";
pub const MAYAN_FORWARDER_CONTRACT: &str = "0x0654874eb7F59C6f5b39931FC45dC45337c967c3";
pub const MAYAN_ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub const SDK_VERSION: &str = "9_7_0";

pub use provider::MayanSwiftProvider;
