use alloy_primitives::hex;
use num_bigint::BigInt;
use num_traits::Num;
use primitives::{EVMChain, TransactionFee, TransactionInputType, TransactionLoadInput};
use serde_serializers::bigint::bigint_from_hex_str;
use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "rpc")]
use crate::rpc::client::EthereumClient;
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::GasPriceType;

use super::preload_mapper::{
    bytes_to_hex_string, get_extra_fee_gas_limit, get_transaction_data, get_transaction_to,
    get_transaction_value,
};

const OPTIMISM_GAS_ORACLE_CONTRACT: &str = "0x420000000000000000000000000000000000000F";

#[cfg(feature = "rpc")]
pub struct OptimismGasOracle<C: Client + Clone> {
    pub chain: EVMChain,
    pub client: EthereumClient<C>,
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> OptimismGasOracle<C> {
    pub fn new(chain: EVMChain, client: EthereumClient<C>) -> Self {
        Self { chain, client }
    }

    pub async fn calculate_fee(&self, input: &TransactionLoadInput, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let data = get_transaction_data(self.chain, input)?;
        let to = get_transaction_to(self.chain, input)?;
        let value = get_transaction_value(self.chain, input)?;

        let nonce = input.metadata.get_sequence()?;
        let chain_id = input.metadata.get_chain_id()?.parse::<u64>()?;

        let extra_gas_limit = get_extra_fee_gas_limit(input)?;

        let adjusted_value = match &input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
                if asset.id.is_native() && input.is_max_value {
                    let parsed_value = BigInt::from_str_radix(&input.value, 10)?;
                    parsed_value - gas_limit * &input.gas_price.gas_price()
                } else {
                    value
                }
            }
            _ => value,
        };

        let encoded = self.encode_transaction_for_l1_fee(
            gas_limit,
            &input.gas_price.gas_price(),
            &input.gas_price.priority_fee(),
            nonce,
            Some(&data),
            &to,
            chain_id,
            Some(&adjusted_value),
            input,
        )?;

        let l1_fee = self.get_l1_fee(&encoded).await?;
        let l2_fee = &input.gas_price.total_fee() * (gas_limit + &extra_gas_limit);

        let fee = l1_fee + l2_fee;

        Ok(TransactionFee::new_gas_price_type(
            GasPriceType::eip1559(input.gas_price.total_fee(), input.gas_price.priority_fee()),
            fee,
            gas_limit.clone(),
            HashMap::new(),
        ))
    }

    async fn get_l1_fee(&self, data: &[u8]) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
        let mut call_data = Vec::with_capacity(4 + 32 + data.len());
        call_data.extend_from_slice(&hex::decode("49948e0e")?);
        call_data.extend_from_slice(&[0u8; 31]);
        call_data.push(0x20);
        let data_len = data.len();
        let len_bytes = BigInt::from(data_len).to_bytes_be().1;
        let padding = 32_usize.saturating_sub(len_bytes.len());
        call_data.extend_from_slice(&vec![0u8; padding]);
        call_data.extend_from_slice(&len_bytes);
        call_data.extend_from_slice(data);
        let data_padding = data.len().div_ceil(32) * 32 - data.len();
        call_data.extend_from_slice(&vec![0u8; data_padding]);

        let result = self.client.eth_call(OPTIMISM_GAS_ORACLE_CONTRACT, &bytes_to_hex_string(&call_data)).await?;

        let result_str: String = result;
        bigint_from_hex_str(&result_str)
    }

    fn encode_transaction_for_l1_fee(
        &self,
        gas_limit: &BigInt,
        gas_price: &BigInt,
        priority_fee: &BigInt,
        nonce: u64,
        call_data: Option<&[u8]>,
        to: &str,
        chain_id: u64,
        value: Option<&BigInt>,
        input: &TransactionLoadInput,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let mut encoded = Vec::new();

        encoded.push(0x02);

        let mut rlp_data = Vec::new();

        rlp_data.extend_from_slice(&chain_id.to_be_bytes());
        rlp_data.extend_from_slice(&nonce.to_be_bytes());
        let priority_bytes = priority_fee.to_bytes_be().1;
        rlp_data.extend_from_slice(&priority_bytes);
        let gas_price_bytes = gas_price.to_bytes_be().1;
        rlp_data.extend_from_slice(&gas_price_bytes);
        let gas_limit_bytes = gas_limit.to_bytes_be().1;
        rlp_data.extend_from_slice(&gas_limit_bytes);
        let to_bytes = hex::decode(to.strip_prefix("0x").unwrap_or(to))?;
        rlp_data.extend_from_slice(&to_bytes);
        if let Some(v) = value {
            let value_bytes = v.to_bytes_be().1;
            rlp_data.extend_from_slice(&value_bytes);
        } else {
            rlp_data.push(0x80);
        }
        if let Some(d) = call_data {
            rlp_data.extend_from_slice(d);
        } else {
            rlp_data.push(0x80);
        }

        rlp_data.push(0xc0);

        encoded.extend_from_slice(&rlp_data);

        match &input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
                if asset.id.is_native() && encoded.len() > 3 {
                    encoded.remove(2);
                }
            }
            _ => {}
        }

        Ok(encoded)
    }
}
