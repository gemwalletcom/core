use alloy_core::{
    hex::{decode as HexDecode, ToHexExt},
    primitives::{Address, U256},
};
use num_traits::ToBytes;
use rand::Rng;
use std::str::FromStr;

use super::{
    forwarder::MayanForwarder,
    models::Quote,
    swift::{MayanSwift, OrderParams},
    MAYAN_FORWARDER_CONTRACT,
};
use crate::{
    config::swap_config::SwapReferralFee,
    swapper::{ApprovalType, SwapQuoteData, SwapperError},
};
use gem_evm::{
    ether_conv,
    mayan::deployment::{get_swift_providers, WormholeId},
};
use primitives::Chain;

#[derive(Default)]
pub struct MayanTxBuilder {}

impl MayanTxBuilder {
    fn generate_random_bytes32() -> [u8; 32] {
        let mut rng = rand::rng();
        let mut random_bytes = [0u8; 32];
        rng.fill(&mut random_bytes);
        random_bytes
    }

    fn address_to_bytes32(address: &str) -> Result<[u8; 32], SwapperError> {
        let addr = Address::from_str(address).map_err(|_| SwapperError::InvalidAddress { address: address.to_string() })?;
        let mut bytes32 = [0u8; 32];
        bytes32[12..].copy_from_slice(addr.as_slice());
        Ok(bytes32)
    }

    fn get_chain_by_wormhole_id(wormhole_id: u64) -> Option<Chain> {
        get_swift_providers()
            .into_iter()
            .find(|(_, deployment)| deployment.wormhole_id.clone() as u64 == wormhole_id)
            .map(|(chain, _)| chain)
    }

    fn to_native_wormhole_address(address: &str, w_chain_id: u64) -> Result<[u8; 32], SwapperError> {
        let chain = Self::get_chain_by_wormhole_id(w_chain_id).ok_or(SwapperError::InvalidRoute)?;
        if chain == Chain::Solana {
            let decoded = bs58::decode(address)
                .into_vec()
                .map_err(|_| SwapperError::InvalidAddress { address: address.to_string() })?;

            let mut bytes32 = [0u8; 32];
            if decoded.len() == 32 {
                bytes32.copy_from_slice(&decoded);
            } else {
                return Err(SwapperError::InvalidAddress {
                    address: format!("Solana address wrong length: {}", address),
                });
            }
            Ok(bytes32)
        } else {
            Self::address_to_bytes32(address)
        }
    }

    fn build_swift_order_params(
        &self,
        quote: &Quote,
        wallet_address: &str,
        destination_address: &str,
        referrer_fee: Option<SwapReferralFee>,
    ) -> Result<OrderParams, SwapperError> {
        let src_chain_id = quote.from_token.w_chain_id;
        let dest_chain_id = quote.to_token.w_chain_id;

        let deadline = quote.deadline64.parse::<u64>().map_err(|_| SwapperError::ComputeQuoteError {
            msg: "Failed to parse deadline".to_string(),
        })?;

        let trader_address = Self::to_native_wormhole_address(wallet_address, src_chain_id)?;
        let destination_address = Self::to_native_wormhole_address(destination_address, dest_chain_id)?;
        let token_out = Self::to_native_wormhole_address(&quote.to_token.contract, dest_chain_id)?;

        let min_amount_out = ether_conv::to_bn_wei(&quote.min_amount_out.to_string(), quote.to_token.decimals as u32).to_string();

        let random_bytes = Self::generate_random_bytes32();

        let referrer_addr: [u8; 32] = referrer_fee
            .as_ref()
            .map(|x| Self::to_native_wormhole_address(x.address.as_str(), WormholeId::Solana as u64))
            .unwrap_or(Ok([0u8; 32]))?;
        let referrer_bps = referrer_fee.map(|x| x.bps).unwrap_or(0);

        let params = OrderParams {
            trader: trader_address,
            token_out,
            min_amount_out: min_amount_out.parse().map_err(|_| SwapperError::InvalidAmount)?,
            gas_drop: 0,
            cancel_fee: quote.cancel_relayer_fee64.unwrap_or(0),
            refund_fee: quote.refund_relayer_fee64.unwrap_or(0),
            deadline,
            dest_addr: destination_address,
            dest_chain_id: dest_chain_id.to_string().parse().map_err(|_| SwapperError::InvalidAmount)?,
            referrer_addr,
            referrer_bps: referrer_bps as u8,
            auction_mode: quote.swift_auction_mode.unwrap_or(0),
            random: random_bytes,
        };

        Ok(params)
    }

    pub fn build_evm_tx(
        &self,
        mayan_quote: Quote,
        approval: ApprovalType,
        wallet_address: &str,
        destination_address: &str,
        referrer_fee: Option<SwapReferralFee>,
    ) -> Result<SwapQuoteData, SwapperError> {
        let mayan_swift_protocol = mayan_quote
            .swift_mayan_contract
            .clone()
            .ok_or(SwapperError::InvalidRoute)?
            .parse::<Address>()
            .unwrap();
        let swift_input_address = mayan_quote
            .swift_input_contract
            .clone()
            .ok_or(SwapperError::InvalidRoute)?
            .parse::<Address>()
            .unwrap();
        let from_token_address = mayan_quote.from_token.contract.clone().parse::<Address>().unwrap();

        let swift_contract = MayanSwift::default();
        let forwarder_contract = MayanForwarder::default();
        let swift_order_params = self.build_swift_order_params(&mayan_quote, wallet_address, destination_address, referrer_fee)?;
        let zero_address = Address::ZERO;

        let mut value = mayan_quote.effective_amount_in64;
        let is_native = swift_input_address == zero_address;
        let need_swap = mayan_quote.evm_swap_router_address.is_some() && mayan_quote.evm_swap_router_calldata.is_some();
        let swift_call_data = if is_native {
            swift_contract.encode_create_order_with_eth(swift_order_params)?
        } else {
            let token_amount = U256::from(mayan_quote.effective_amount_in64);
            swift_contract.encode_create_order_with_token(from_token_address, token_amount, swift_order_params)?
        };

        let amount = U256::from(mayan_quote.effective_amount_in64);

        let forwarder_call_data = if !need_swap {
            if is_native {
                forwarder_contract.encode_forward_eth_call(mayan_swift_protocol, swift_call_data.clone())?
            } else {
                value = 0;
                forwarder_contract.encode_forward_erc20_call(from_token_address, amount, None, mayan_swift_protocol, swift_call_data.clone())?
            }
        } else {
            let evm_swap_router_address = mayan_quote.evm_swap_router_address.unwrap();
            let evm_swap_router_calldata = mayan_quote.evm_swap_router_calldata.unwrap();
            let min_middle_amount = mayan_quote.min_middle_amount.unwrap();

            let min_middle_amount_bigint = ether_conv::to_bn_wei(&min_middle_amount.to_string(), mayan_quote.swift_input_decimals as u32);
            let amount_in = U256::from(mayan_quote.effective_amount_in64);
            let swap_data = HexDecode(evm_swap_router_calldata).map_err(SwapperError::from)?;
            let min_middle_amount = U256::from_le_slice(min_middle_amount_bigint.to_le_bytes().as_slice());

            let swap_protocol = Address::from_str(&evm_swap_router_address).map_err(|_| SwapperError::InvalidAddress {
                address: evm_swap_router_address.to_string(),
            })?;
            let middle_token_address = swift_input_address;

            if is_native {
                forwarder_contract.encode_swap_and_forward_eth_call(
                    amount_in,
                    swap_protocol,
                    swap_data,
                    middle_token_address,
                    min_middle_amount,
                    mayan_swift_protocol,
                    swift_call_data,
                )?
            } else {
                value = 0;

                forwarder_contract.encode_swap_and_forward_erc20_call(
                    from_token_address,
                    amount_in,
                    None,
                    swap_protocol,
                    swap_data,
                    middle_token_address,
                    min_middle_amount,
                    mayan_swift_protocol,
                    swift_call_data,
                )?
            }
        };

        // FIXME: add gas limit
        Ok(SwapQuoteData {
            to: MAYAN_FORWARDER_CONTRACT.to_string(),
            value: value.to_string(),
            data: forwarder_call_data.encode_hex(),
            approval: approval.approval_data(),
            gas_limit: None,
        })
    }

    pub fn build_sol_tx(&self, _quote: Quote) -> Result<SwapQuoteData, SwapperError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::swapper::mayan::models::QuoteResponse;

    pub fn generate_mock_quote() -> Vec<Quote> {
        let data = include_str!("test/quote_response.json");
        let response: QuoteResponse = serde_json::from_str(data).expect("Failed to deserialize Quote");
        response.quotes
    }

    #[test]
    fn test_address_to_bytes32_valid() {
        let address = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let bytes32 = MayanTxBuilder::address_to_bytes32(address).unwrap();
        let expected_bytes32 = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 85, 198, 171, 218, 94, 42, 82, 65, 170, 8, 72, 107, 213, 12, 247, 212, 117, 207, 36,
        ];

        assert_eq!(bytes32, expected_bytes32);
    }

    #[test]
    fn test_build_swift_order_params_valid() {
        let provider = MayanTxBuilder::default();
        let quotes = generate_mock_quote();
        let quote = quotes.first().unwrap();
        let wallet_address = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let destination_address = "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR";
        let referrer_fee = Some(SwapReferralFee {
            address: "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24".to_string(),
            bps: 100,
        });

        let result = provider
            .build_swift_order_params(quote, wallet_address, destination_address, referrer_fee)
            .inspect_err(|e| {
                eprintln!("Failed to build swift order params: {}", e);
            });

        assert!(result.is_ok())
    }

    #[test]
    fn test_quote_serialization_deserialization() {
        let quotes = generate_mock_quote();
        let quote = quotes.first().unwrap();

        // Serialize the quote
        let serialized = serde_json::to_string(&quote).expect("Failed to serialize quote");

        // Deserialize the quote
        let deserialized: Quote = serde_json::from_str(&serialized).expect("Failed to deserialize quote");

        // Verify key fields match
        assert_eq!(quote.r#type, deserialized.r#type);
        assert_eq!(quote.from_token.name, deserialized.from_token.name);
        assert_eq!(quote.from_token.contract.to_string(), deserialized.from_token.contract.to_string());
        assert_eq!(quote.to_token.name, deserialized.to_token.name);
        assert_eq!(quote.to_token.contract.to_string(), deserialized.to_token.contract.to_string());
        assert_eq!(quote.min_amount_out, deserialized.min_amount_out);
        assert_eq!(quote.swift_auction_mode, deserialized.swift_auction_mode);
    }
}
