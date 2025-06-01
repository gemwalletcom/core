use async_trait::async_trait;
use primitives::{name::NameProvider, Chain, Address}; // Added Address
use std::error::Error;
use anyhow::Result; // Added for Result type

use crate::client::NameClient;
// No longer need to add `use crate::hyperliquid_provider::Provider;` here as it's added below with the mod.

// ROUTER_ABI and REGISTRATOR_ABI constants removed as they are now defined in hyperliquid_provider.rs

mod hyperliquid_provider;
use crate::hyperliquid_provider::Provider; // Added Provider import here after mod declaration

#[derive(Debug)] // Added Debug derive
pub struct HLNamesClient {
    provider: Provider,
}

impl HLNamesClient {
    pub fn new(rpc_url: String) -> Result<Self> { // Updated signature and return type
        let provider = Provider::new(rpc_url)?;
        Ok(Self { provider })
    }

    pub fn is_valid_name(&self, name: &str) -> bool {
        let labels = name.split('.').collect::<Vec<&str>>();
        if labels.is_empty() {
            return false;
        }

        !labels.iter().any(|label| label.is_empty())
    }

    // The old resolve_name method is now removed.
}

#[async_trait]
impl NameClient for HLNamesClient {
    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Hyperliquid]
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        if !self.chains().contains(&chain) {
            // Updated error message to include provider ID
            return Err(format!("Unsupported chain: {} for provider {}", chain, self.provider().id()).into());
        }

        if !self.is_valid_name(name) {
            return Err(format!("Invalid name: {}", name).into());
        }

        match self.provider.resolve_name(name, chain).await {
            Ok(address) => {
                // Assuming Address from primitives has a to_string() or Display trait
                // that converts it to a hex string.
                Ok(address.to_string())
            }
            Err(e) => {
                // Log the error or handle it as needed
                // e.g., log::error!("Failed to resolve name {} on chain {}: {:?}", name, chain, e);
                Err(format!("Failed to resolve name {} on chain {}: {}", name, chain, e).into())
            }
        }
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Hyperliquid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["hl"]
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Imports HLNamesClient, NameClient, Chain etc.
    use primitives::Chain; // Explicitly import if not covered by super::* or needed

    #[test]
    fn test_is_valid_name() {
        // Update HLNamesClient instantiation
        // Using a dummy RPC URL as this test does not make network calls.
        let client = HLNamesClient::new("http://localhost:1234".to_string())
            .expect("Failed to create HLNamesClient for test");

        assert!(client.is_valid_name("test.hl"));
        assert!(client.is_valid_name("a.test.hl"));
        assert!(client.is_valid_name("a.b.test.hl"));
        assert!(client.is_valid_name("foo-bar.hl"));
        assert!(client.is_valid_name("123.hl"));

        assert!(!client.is_valid_name("test..hl")); // Empty label
        assert!(!client.is_valid_name("test.hl.")); // Trailing dot
        assert!(!client.is_valid_name(".test.hl")); // Leading dot on label
        assert!(!client.is_valid_name("test.hl.."));
        assert!(!client.is_valid_name("test.hl..hl"));
        assert!(!client.is_valid_name("")); // Empty name
        assert!(!client.is_valid_name(".hl")); // Only TLD with leading dot
        assert!(!client.is_valid_name("hl")); // Only TLD
        assert!(!client.is_valid_name("test.foo")); // Invalid TLD
    }

    // TODO: Expand tests for resolve method with proper mocking or integration setup,
    // and once the actual Hyperliquid namehashing algorithm is implemented in the provider.

    #[tokio::test]
    async fn test_resolve_invalid_name() {
        let client = HLNamesClient::new("http://localhost:1234".to_string())
            .expect("Failed to create HLNamesClient for test");
        let result = client.resolve("test.hl..", Chain::Hyperliquid).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid name"));
        }
    }

    #[tokio::test]
    async fn test_resolve_unsupported_chain() {
        let client = HLNamesClient::new("http://localhost:1234".to_string())
            .expect("Failed to create HLNamesClient for test");
        // Assuming Ethereum is not Hyperliquid for this test
        let result = client.resolve("test.hl", Chain::Ethereum).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Unsupported chain"));
        }
    }

    #[tokio::test]
    async fn test_resolve_network_error_or_placeholder_hash() {
        // This test uses a dummy RPC URL which should cause a network error,
        // or if it somehow connected, the placeholder namehash would likely not resolve.
        let client = HLNamesClient::new("http://invalid-dummy-rpc-url-for-test:12345".to_string())
            .expect("Failed to create HLNamesClient for test");

        let result = client.resolve("test.hl", Chain::Hyperliquid).await;
        assert!(result.is_err(), "Resolve should fail for a dummy RPC or placeholder hash");
        // We can't be too specific about the error message here as it could be a network error
        // or an error from the contract if the placeholder hash doesn't exist.
        // Example: "Failed to resolve name test.hl on chain hyperliquid: ..."
        if let Err(e) = result {
             println!("Resolve error (expected): {}", e); // Optional: print for debugging
             assert!(e.to_string().starts_with("Failed to resolve name test.hl on chain hyperliquid:"));
        }
    }
}
