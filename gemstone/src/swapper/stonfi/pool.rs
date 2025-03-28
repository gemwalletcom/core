use std::str::FromStr;

use alloy_primitives::U256;
use base64::engine::general_purpose::STANDARD;
use base64_serde::base64_serde_type;
use num_bigint::{BigInt, BigUint};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tonlib_client::tl::{TvmCell, TvmNumber, TvmStack, TvmStackEntry};
use tonlib_core::TonAddress;

use num_traits::Num;

use crate::swapper::slippage::apply_slippage_in_bp;
base64_serde_type!(Base64Standard, STANDARD);

fn hex_to_bigint(hex_str: &str) -> BigInt {
    let cleaned_hex = hex_str.trim_start_matches("0x"); // Remove "0x" if present
    BigInt::from_str_radix(cleaned_hex, 16).expect("Invalid hex string")
}

fn bigint_to_string(big_int: &BigInt) -> String {
    big_int.to_string() // Converts back to decimal string
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum StackEntry {
    #[serde(rename = "num")]
    Number {
        #[serde(rename = "value")]
        number: String,
    },
    #[serde(rename = "cell")]
    Cell {
        #[serde(rename = "value", with = "Base64Standard")]
        cell: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct TonRequest {
    address: String,
    method: String,
    stack: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TonResponse {
    ok: bool,
    result: RunMethodResult,
}

#[derive(Debug, Serialize, Deserialize)]
struct RunMethodResult {
    gas_used: u32,
    stack: Vec<StackEntry>,
    exit_code: i32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PoolData {
    is_locked: bool,
    router_address: TonAddress,
    total_supply_lp: BigUint,
    reserve0: BigUint,
    reserve1: BigUint,
    token0_wallet_address: TonAddress,
    token1_wallet_address: TonAddress,
    lp_fee: BigUint,
    protocol_fee: BigUint,
    protocol_fee_address: Option<TonAddress>,
    collected_token0_protocol_fee: BigUint,
    collected_token1_protocol_fee: BigUint,
}

pub async fn get_pool_data(client: &Client, contract_address: &str) -> Result<PoolData, Box<dyn std::error::Error>> {
    let request_body = TonRequest {
        address: contract_address.to_string(),
        method: "get_pool_data".to_string(),
        stack: vec![],
    };

    println!("request_body: {:?}", request_body);

    let response = client.post("https://toncenter.com/api/v3/runGetMethod").json(&request_body).send().await?;

    println!("response: {:?}", response);

    let data = response.json::<RunMethodResult>().await?;

    println!("data: {:?}", data);

    let stack_elements = data
        .stack
        .iter()
        .map(|x| match x {
            StackEntry::Number { number } => TvmStackEntry::Number {
                number: TvmNumber {
                    number: bigint_to_string(&hex_to_bigint(number)),
                },
            },
            StackEntry::Cell { cell } => TvmStackEntry::Cell {
                cell: TvmCell { bytes: cell.clone() },
            },
        })
        .collect::<Vec<TvmStackEntry>>();

    println!("stack_elements: {:?}", stack_elements);

    let stack = TvmStack::from(stack_elements.as_slice());

    println!("stack: {:?}", stack);

    let is_locked = stack.get_i32(0)? != 0;
    let is_locked = false;
    let router_address = stack.get_address(1)?;
    let total_supply_lp = stack.get_biguint(2)?;
    let reserve0 = stack.get_biguint(3)?;
    let reserve1 = stack.get_biguint(4)?;
    let token0_wallet_address = stack.get_address(5)?;
    let token1_wallet_address = stack.get_address(6)?;
    let lp_fee = stack.get_biguint(7)?;
    let protocol_fee = stack.get_biguint(8)?;
    let protocol_fee_address = stack.get_address(9).ok(); // Optional
    let collected_token0_protocol_fee = stack.get_biguint(10)?;
    let collected_token1_protocol_fee = stack.get_biguint(11)?;
    println!("total_supply_lp: {:?}", total_supply_lp);

    Ok(PoolData {
        is_locked,
        router_address,
        total_supply_lp,
        reserve0,
        reserve1,
        token0_wallet_address,
        token1_wallet_address,
        lp_fee,
        protocol_fee,
        protocol_fee_address,
        collected_token0_protocol_fee,
        collected_token1_protocol_fee,
    })
}

pub fn calculate_tokens_out(amount: String, referrer_bps: u32, pool_data: &PoolData) -> Result<String, Box<dyn std::error::Error>> {
    let amount_in = BigUint::from_str(&amount)?;
    let fees = BigUint::from(referrer_bps) + &pool_data.lp_fee + &pool_data.protocol_fee;
    let fees_u32 = fees.clone().to_u32_digits().first().unwrap();
    let effective_amount_with_fees = apply_slippage_in_bp(&U256::from_str(amount.as_str())?, fees_u32.clone());
    let effective_amount = BigUint::from_str(effective_amount_with_fees.to_string().as_str())?;
    let amount_out = (&effective_amount * &pool_data.reserve0) / (&pool_data.reserve1 + &effective_amount);
    Ok(amount_out.to_string())
}

#[cfg(test)]
mod tests {
    use num_traits::Num;

    use super::*;

    #[test]
    fn test_bigint() {
        let stack = TvmStack::from(&[
            TvmStackEntry::Number {
                number: TvmNumber { number: "100500".to_string() },
            },
            TvmStackEntry::Number {
                number: TvmNumber {
                    number: bigint_to_string(&hex_to_bigint("0xbb96c6b5088c")),
                },
            },
        ]);
        println!("test {:?}", BigUint::from_str_radix("bb96c6b5088c", 16).unwrap());
        assert_eq!(BigInt::from(100500), stack.get_bigint(0).unwrap());
        assert_eq!(BigUint::from_str_radix("bb96c6b5088c", 16).unwrap(), stack.get_biguint(1).unwrap());
    }
}
