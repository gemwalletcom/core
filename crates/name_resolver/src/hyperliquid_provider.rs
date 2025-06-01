use alloy_primitives::{Address as AlloyAddress, FixedBytes, U256};
use alloy_provider::{Provider as AlloyEthersProvider, ProviderBuilder, RootProvider}; // Renamed to avoid conflict
use alloy_rpc_client::RpcClient;
use alloy_sol_types::{sol, SolCall, SolInterface};
use alloy_transport_http::Http;
use primitives::{Chain, Address as PrimitiveAddress}; // Renamed to avoid conflict
use reqwest::Client;
use std::str::FromStr;
use anyhow::Result;
use alloy_ens::fns::namehash; // Added import

// TODO: Confirm if Hyperliquid uses the exact ENS namehashing algorithm.
// Assuming it does for now based on the suggestion to use alloy-ens.
fn hyperliquid_namehash(name: &str) -> FixedBytes<32> {
    namehash(name)
}

// Define the contract ABIs using alloy_sol_types
sol! {
    #[sol(rpc)]
    interface RouterContract {
        function getCurrentRegistrator() external view returns (address);
    }

    #[sol(rpc)]
    interface RegistratorContract {
        // alloy_sol_types uses bytes32 for FixedBytes<32>
        function ownerOf(bytes32 _namehash) external view returns (address);
    }
}

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";

#[derive(Debug)]
pub struct Provider {
    // Use alloy's RootProvider or a specific provider type
    ethers_provider: RootProvider<Http<Client>>,
}

impl Provider {
    pub fn new(rpc_url_str: String) -> Result<Self> {
        let rpc_url = rpc_url_str.parse()?;
        let http_client = Client::new(); // Create a reqwest client
        let transport = Http::new_with_client(rpc_url, http_client); // Pass the client to Http transport
        let provider = ProviderBuilder::new().on_http(transport);
        Ok(Self {
            ethers_provider: provider,
        })
    }

    pub async fn resolve_name(&self, name: &str, _chain: Chain) -> Result<PrimitiveAddress> {
        // TODO: Add validation for the name format if necessary

        let router_address = AlloyAddress::from_str(ROUTER_ADDRESS)
            .map_err(|e| anyhow::anyhow!("Failed to parse Router address: {}", e))?;

        // Create the call for getCurrentRegistrator
        let call_builder = RouterContract::getCurrentRegistratorCall {};
        let call_request = call_builder.create_request(Some(router_address), None, None);

        // Make the call
        let result_bytes = self.ethers_provider
            .call(&call_request)
            .await?;

        let registrator_address = RouterContract::getCurrentRegistratorCall::abi_decode_returns(&result_bytes, false)?.element;


        // Generate the namehash for the input name
        let namehash = hyperliquid_namehash(name);

        // Create the call for ownerOf
        let owner_of_call_builder = RegistratorContract::ownerOfCall { _namehash: namehash };
        let owner_of_request = owner_of_call_builder.create_request(Some(registrator_address), None, None);


        // Make the call to the registrator contract
        let owner_result_bytes = self.ethers_provider
            .call(&owner_of_request)
            .await?;

        let owner_address = RegistratorContract::ownerOfCall::abi_decode_returns(&owner_result_bytes, false)?.element;

        // Convert alloy_primitives::Address to your PrimitiveAddress
        // Assuming PrimitiveAddress can be created from a byte slice or [u8; 20]
        Ok(PrimitiveAddress::from(owner_address.as_slice()))
    }
}
