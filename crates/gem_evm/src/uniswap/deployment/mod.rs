pub mod v3;
pub mod v4;

use primitives::{Chain, SwapProvider};

pub trait Deployment {
    fn quoter(&self) -> &'static str;
    fn permit2(&self) -> &'static str;
    fn universal_router(&self) -> &'static str;
}

pub fn get_uniswap_permit2_by_chain(chain: &Chain) -> Option<&'static str> {
    match chain {
        Chain::Ethereum
        | Chain::Optimism
        | Chain::Arbitrum
        | Chain::Polygon
        | Chain::AvalancheC
        | Chain::Base
        | Chain::SmartChain
        | Chain::Celo
        | Chain::Blast
        | Chain::World
        | Chain::Unichain
        | Chain::Ink => Some("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
        Chain::ZkSync | Chain::Abstract => Some("0x0000000000225e31d15943971f47ad3022f714fa"),
        _ => None,
    }
}

pub fn get_provider_by_chain_contract(chain: &Chain, contract: &str) -> Option<String> {
    if let Some(deployment) = v3::get_uniswap_router_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::UniswapV3.id().to_string());
        }
    }
    if let Some(deployment) = v4::get_uniswap_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::UniswapV4.id().to_string());
        }
    }
    if let Some(deployment) = v3::get_pancakeswap_router_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::PancakeswapV3.id().to_string());
        }
    }
    if let Some(deployment) = v3::get_oku_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::Oku.id().to_string());
        }
    }
    if let Some(deployment) = v3::get_wagmi_router_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::Wagmi.id().to_string());
        }
    }
    if let Some(deployment) = v3::get_reservoir_deployment_by_chain(chain) {
        if deployment.universal_router.to_lowercase() == contract.to_lowercase() {
            return Some(SwapProvider::Reservoir.id().to_string());
        }
    }
    None
}
