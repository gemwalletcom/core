use crate::WalletConnectionSessionAppMetadata;

impl WalletConnectionSessionAppMetadata {
    pub fn mock() -> Self {
        WalletConnectionSessionAppMetadata {
            name: "test".into(),
            description: "test".into(),
            url: "https://test.com".into(),
            icon: "".into(),
        }
    }
}
