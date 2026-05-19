use crate::{SwapperError, mayan::wormhole_chain};
use gem_evm::address::EthereumAddress;
use gem_solana::SolanaAddress;
use gem_sui::address::SuiAddress;
use primitives::{Address as AddressTrait, ChainType, decode_hex_array};

pub(super) fn native_address_to_bytes32(address: &str, wormhole_chain_id: u16) -> Result<[u8; 32], SwapperError> {
    let chain = wormhole_chain::chain_from_id(wormhole_chain_id).ok_or(SwapperError::NotSupportedChain)?;
    match chain.chain_type() {
        ChainType::Solana => address_bytes::<SolanaAddress, 32>(address, "Solana"),
        ChainType::Ethereum => evm_address_bytes32(address),
        ChainType::Sui => address_bytes::<SuiAddress, 32>(address, "Sui"),
        ChainType::Ton => decode_hex_array::<32>(address).map_err(SwapperError::from),
        _ => Err(SwapperError::NotSupportedChain),
    }
}

pub(super) fn evm_address_bytes(address: &str) -> Result<[u8; 20], SwapperError> {
    address_bytes::<EthereumAddress, 20>(address, "EVM")
}

fn evm_address_bytes32(address: &str) -> Result<[u8; 32], SwapperError> {
    let bytes = evm_address_bytes(address)?;
    let mut padded = [0u8; 32];
    padded[12..].copy_from_slice(&bytes);
    Ok(padded)
}

fn address_bytes<A: AddressTrait, const N: usize>(address: &str, chain: &str) -> Result<[u8; N], SwapperError> {
    A::from_str(address)
        .map_err(|err| SwapperError::ComputeQuoteError(format!("Invalid {chain} address: {err}")))?
        .as_bytes()
        .try_into()
        .map_err(|_| SwapperError::ComputeQuoteError(format!("Invalid {chain} address length")))
}
