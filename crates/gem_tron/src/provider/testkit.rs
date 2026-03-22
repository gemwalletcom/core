#[cfg(test)]
use crate::models::account::{TronAccount, TronAccountOwnerPermission, TronAccountPermission, TronAccountPermissionKey, TronFrozen, TronVote};
#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::client::TronClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use primitives::asset_constants::TRON_USDT_TOKEN_ID;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "TFdTEn9dJuqh351y8fyJ3eMmghFsZNwakb";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_USDT_TOKEN_ID: &str = TRON_USDT_TOKEN_ID;

#[cfg(test)]
impl TronAccount {
    pub fn mock(address: &str) -> Self {
        Self {
            balance: None,
            address: Some(address.to_string()),
            owner_permission: Some(TronAccountOwnerPermission {
                permission_name: "owner".to_string(),
                threshold: Some(1),
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
            }),
            active_permission: Some(vec![TronAccountPermission {
                id: None,
                threshold: 1,
                keys: Some(vec![TronAccountPermissionKey {
                    address: address.to_string(),
                    weight: 1,
                }]),
            }]),
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        }
    }

    pub fn mock_with_staking(votes: Option<Vec<TronVote>>, frozen_v2: Option<Vec<TronFrozen>>) -> Self {
        Self {
            balance: None,
            address: None,
            owner_permission: None,
            active_permission: None,
            votes,
            frozen_v2,
            unfrozen_v2: None,
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_test_client() -> TronClient<ReqwestClient> {
    use crate::rpc::trongrid::client::TronGridClient;
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.tron.url, reqwest::Client::new());
    let trongrid_client = TronGridClient::new(reqwest_client.clone(), settings.trongrid.key.secret);
    TronClient::new(reqwest_client, trongrid_client)
}
