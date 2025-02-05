use primitives::Chain;
pub mod v3;
pub mod v4;

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
        | Chain::World => Some("0x000000000022D473030F116dDEE9F6B43aC78BA3"),
        Chain::ZkSync | Chain::Abstract => Some("0x0000000000225e31d15943971f47ad3022f714fa"),
        _ => None,
    }
}
