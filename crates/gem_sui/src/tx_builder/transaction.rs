use super::TransactionBuilderInput;
use crate::{
    SuiError, is_sui_coin,
    models::{CoinAsset, TxOutput},
};
use gem_encoding::decode_base64;
use num_traits::ToPrimitive;
use serde::de::DeserializeOwned;
use std::{error::Error, str::FromStr};
use sui_transaction_builder::{Argument, Function, TransactionBuilder};
use sui_types::{Address, Identifier, TypeTag};

const MODULE_COIN: &str = "coin";
const FUNCTION_ZERO: &str = "zero";

pub fn move_call(txb: &mut TransactionBuilder, package: &str, module: &str, function: &str, type_args: &[&str], arguments: Vec<Argument>) -> Result<Argument, SuiError> {
    let type_args = type_args
        .iter()
        .map(|value| {
            value
                .parse::<TypeTag>()
                .map_err(|err| SuiError::invalid_input(format!("Invalid Sui type argument {value}: {err}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let function = Function::new(
        Address::from_str(package).map_err(|err| SuiError::invalid_input(format!("Invalid Sui address {package}: {err}")))?,
        Identifier::new(module).map_err(|err| SuiError::invalid_input(err.to_string()))?,
        Identifier::new(function).map_err(|err| SuiError::invalid_input(err.to_string()))?,
    )
    .with_type_args(type_args);
    Ok(txb.move_call(function, arguments))
}

pub fn zero_coin(txb: &mut TransactionBuilder, coin_type: &str) -> Result<Argument, SuiError> {
    move_call(txb, "0x2", MODULE_COIN, FUNCTION_ZERO, &[coin_type], vec![])
}

pub fn build_input_coin(txb: &mut TransactionBuilder, coin_type: &str, amount: u64, from_coins: &[CoinAsset]) -> Result<Argument, SuiError> {
    if amount == 0 {
        return zero_coin(txb, coin_type);
    }

    if is_sui_coin(coin_type) {
        let amount = txb.pure(&amount);
        let gas = txb.gas();
        return txb.split_coins(gas, vec![amount]).pop().ok_or_else(|| SuiError::invalid_input("Sui split coin failed"));
    }

    let total = from_coins.iter().try_fold(0_u64, |total, coin| {
        let balance = coin.balance.to_u64().ok_or_else(|| SuiError::invalid_input("Sui coin balance overflow"))?;
        total.checked_add(balance).ok_or_else(|| SuiError::invalid_input("Sui coin balance overflow"))
    })?;
    if total < amount {
        return Err(SuiError::InsufficientBalance);
    }

    let mut coin_args = from_coins.iter().map(|coin| Ok(txb.object(coin.to_input()))).collect::<Result<Vec<_>, SuiError>>()?;
    let coin = coin_args.first().copied().ok_or(SuiError::InsufficientBalance)?;
    if coin_args.len() > 1 {
        txb.merge_coins(coin, coin_args.split_off(1));
    }

    let amount = txb.pure(&amount);
    txb.split_coins(coin, vec![amount]).pop().ok_or_else(|| SuiError::invalid_input("Sui split coin failed"))
}

pub fn finish_transaction(mut txb: TransactionBuilder, input: TransactionBuilderInput) -> Result<TxOutput, SuiError> {
    txb.set_sender(Address::from_str(&input.sender).map_err(|err| SuiError::invalid_input(format!("Invalid Sui address {}: {err}", input.sender)))?);
    txb.set_gas_price(input.gas_price);
    txb.set_gas_budget(input.gas_budget);
    txb.add_gas_objects(input.gas_objects);

    let tx = txb.try_build().map_err(|err| SuiError::invalid_input(err.to_string()))?;
    TxOutput::from_tx(&tx).map_err(|err| SuiError::invalid_input(err.to_string()))
}

pub fn decode_transaction<T: DeserializeOwned>(encoded: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
    let bytes = decode_base64(encoded)?;
    let transaction = bcs::from_bytes::<T>(&bytes)?;
    Ok(transaction)
}

pub fn validate_and_hash(encoded: &str) -> Result<TxOutput, Box<dyn Error + Send + Sync>> {
    if encoded.trim().is_empty() {
        return Err("Missing Sui transaction data".into());
    }

    let transaction = decode_transaction(encoded).map_err(|err| format!("Invalid Sui transaction payload: {err}"))?;
    TxOutput::from_tx(&transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sui_types::{Transaction, TransactionKind};

    #[test]
    fn test_decode_transaction() {
        let encoded = "AAAPAAhkx5NBAAAAAAAIKUO8sgMAAAAAAQAAAQAAAQAACGTHk0EAAAAAAQFexM/GvrUlJRacMqd+FsKIt7/Lm4mCielL8xCFcLPvpBbjZwAAAAAAAQEB2qRikmMsPE2PMfI+oPmzaij/NnfpaEmA5EOEA6Z6PY8uBRgAAAAAAAABAYBJ0AkRYmmsBO4UIGt6/YtktYASefhUAe5LOXefgJE0zicvAAAAAAABAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAEB8ZTZsbytly5Fp91n3Umz7h4zV6AKUIUMUs1Ru0UOE7QXwmUAAAAAAAABASjkmd/16GSi6v5HYmmk9QNfHBbzONp74YsQNJmr8nHO7fIyAAAAAAABAQHwxA1nsHgADhgDIzTDMlxHueyfPZrkEovoINVGY9FOO+/yMgAAAAAAAQEBNdNbDlsXdZPYw6gBRiSFVy/DCGHmzpalWvbcRzBwknju8jIAAAAAAAAAIJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2BgIAAQEAAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFvYnRhaW5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAUCAAABAQABAgABAwABBAAA3BVyG6qCumSCLVhac0mhUI922UroDombBuSDacJXdQ4Ic3dhcF9jYXANaW5pdGlhdGVfcGF0aAEHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQACAgEAAQUAAB7GqMWsC4uXwofNNLn8apS1OgfJMKhQWVJnncjUs3gKBnJvdXRlchBzd2FwX2JfdG9fYV9ieV9iAwcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgNzdWkDU1VJAAfkI5zZUfbFPZxB4lJw2A0x+SWtFlXlultUOEPUpml17gRTVUlQBFNVSVAABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkABgEGAAIBAAEHAAEIAAICAAEJAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFyZXR1cm5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAYCAQACAwABCgABCwABDAABDQABAQIDAAEOAJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2AQAX1Cs2B1S8591qpdZjDUOB/CBDy2V8/6tqhBbwbdyxj734BAAAAAAg6yrtiW5R0TC68GDMmZye6U+KDjfZlq21n3bztRGzXjuT9luMFsJjNDu/Zs+fju9pyx28ktE/DDMbDcrrdrSqtu4CAAAAAAAA3P9fAAAAAAAA";
        let transaction: Transaction = decode_transaction(encoded).unwrap();

        assert_eq!(transaction.sender.to_string(), "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        match transaction.kind {
            TransactionKind::ProgrammableTransaction(programmable) => {
                assert_eq!(programmable.commands.len(), 6);
            }
            _ => panic!("wrong kind"),
        }

        let output = validate_and_hash(encoded).unwrap();
        assert_eq!(hex::encode(output.hash), "883f6f54145fdaf357e3d404a8353b1f6eda265bc2b28ec8178631e092c24e3b");
    }
}
