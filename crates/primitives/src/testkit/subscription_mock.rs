use crate::{Chain, Device, DeviceSubscription, WalletId};

impl DeviceSubscription {
    pub fn mock() -> Self {
        Self {
            wallet_row_id: 1,
            device: Device::mock(),
            wallet_id: WalletId::Multicoin("0xABC".to_string()),
            chain: Chain::Ethereum,
            address: "0xABC".to_string(),
        }
    }
}
