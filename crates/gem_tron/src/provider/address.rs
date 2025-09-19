use async_trait::async_trait;
use chain_traits::ChainAddressStatus;
use gem_client::Client;
use primitives::AddressStatus;
use std::error::Error;

use crate::provider::address_mapper;
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainAddressStatus for TronClient<C> {
    async fn get_address_status(&self, address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(address_mapper::map_address_status(&account))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {

    use super::*;
    use crate::provider::testkit::{create_test_client, TEST_ADDRESS};

    #[tokio::test]
    async fn test_get_address_status_regular() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let status = client.get_address_status(TEST_ADDRESS.to_string()).await?;

        assert!(status.is_empty());

        let status = client.get_address_status("TYeyZXywpA921LEtw2PF3obK4B8Jjgpp32".to_string()).await?;

        assert!(status.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_multi_signature() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let status = client.get_address_status("TDTcR8wBLadFYRekvobSSswHaj351EDNRT".to_string()).await?;

        println!("Status: {:?}", status);

        assert!(
            status.contains(&AddressStatus::MultiSignature),
            "Expected multi-signature status for known multi-sig wallet"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_multi_signature_owner() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let status = client.get_address_status("THzbnFasHU6AsHfbKahznBNC3Ss591zwPS".to_string()).await?;

        println!("Status: {:?}", status);

        assert!(
            status.contains(&AddressStatus::MultiSignature),
            "Expected multi-signature status for known multi-sig wallet"
        );

        Ok(())
    }
}
