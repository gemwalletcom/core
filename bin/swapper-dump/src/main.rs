use gemstone::{network::alien_provider::NativeProvider, swapper::GemSwapper};
use primitives::Chain;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::Arc,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let network_provider = Arc::new(NativeProvider::default());
    let swapper = GemSwapper::new(network_provider);
    let providers = swapper.swappers;

    // Create maps for both perspectives
    let mut provider_chains: HashMap<String, HashSet<Chain>> = HashMap::new();
    let mut chain_providers: HashMap<Chain, HashSet<String>> = HashMap::new();

    for provider in &providers {
        let provider_type = provider.provider();
        let supported_chains: HashSet<Chain> = provider.supported_assets().iter().map(|x| x.get_chain()).collect();

        // Add to provider perspective
        provider_chains
            .entry(provider_type.name.to_string())
            .or_default()
            .extend(supported_chains.clone());

        // Add to chain perspective
        for chain in supported_chains {
            chain_providers.entry(chain).or_default().insert(provider_type.name.to_string());
        }
    }

    // Print Provider perspective
    println!("==================================");
    println!("Provider -> Chains:");
    println!("==================================");

    let mut sorted_providers: Vec<_> = provider_chains.keys().collect();
    sorted_providers.sort();

    for provider in sorted_providers {
        if let Some(chains) = provider_chains.get(provider) {
            let mut sorted_chains: Vec<_> = chains.iter().collect();
            sorted_chains.sort_by_key(|chain| format!("{:?}", chain));
            println!("{}({}):", provider, chains.len());
            for chain in sorted_chains {
                println!("    {:?}", chain);
            }
            println!();
        }
    }

    // Print Chain perspective
    println!("==================================");
    println!("Chain -> Providers:");
    println!("==================================");

    let mut chains: Vec<_> = chain_providers.keys().collect();
    chains.sort_by_key(|chain| format!("{:?}", chain));

    for chain in chains {
        if let Some(providers) = chain_providers.get(chain) {
            let mut sorted_providers: Vec<_> = providers.iter().collect();
            sorted_providers.sort();
            println!("{:?}:", chain);
            for provider in sorted_providers {
                println!("    {}", provider);
            }
            println!();
        }
    }

    Ok(())
}
