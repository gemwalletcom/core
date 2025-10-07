use std::sync::Arc;

use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use primitives::Chain;

use crate::alien::{AlienClient, AlienError, AlienProvider, new_alien_client};

pub fn tron_client(provider: Arc<dyn AlienProvider>) -> Result<TronClient<AlienClient>, AlienError> {
    let endpoint = provider.get_endpoint(Chain::Tron)?;
    let tron_rpc_client = new_alien_client(endpoint.clone(), provider.clone());
    let trongrid_client = TronGridClient::new(new_alien_client(endpoint, provider), String::new());
    Ok(TronClient::new(tron_rpc_client, trongrid_client))
}
