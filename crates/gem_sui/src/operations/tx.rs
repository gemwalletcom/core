use anyhow::Error;
use base64::{engine::general_purpose, Engine as _};
use serde::de::DeserializeOwned;
use sui_transaction_builder::{unresolved::Input, TransactionBuilder};
use sui_types::{Address, ObjectDigest, ObjectId, Transaction};

use crate::{models::TxOutput, EMPTY_ADDRESS};

pub fn decode_transaction<T: DeserializeOwned>(tx: &str) -> Result<T, Error> {
    let bytes = general_purpose::STANDARD.decode(tx)?;
    let tx = bcs::from_bytes::<T>(&bytes)?;
    Ok(tx)
}

pub fn validate_and_hash(encoded: &str) -> Result<TxOutput, Error> {
    let tx_data = decode_transaction(encoded)?;
    TxOutput::from_tx(&tx_data)
}

pub fn prefill_tx(ptb: TransactionBuilder) -> Transaction {
    let mut ptb = ptb;
    let empty_address = EMPTY_ADDRESS.parse::<Address>().unwrap();
    ptb.set_sender(empty_address);
    ptb.set_gas_price(0);
    ptb.set_gas_budget(0);
    ptb.add_gas_objects(vec![Input::owned(ObjectId::from(empty_address), 0, ObjectDigest::ZERO)]);
    ptb.finish().unwrap()
}

pub fn fill_tx(ptb: &mut TransactionBuilder, sender_address: Address, gas_price: u64, gas_budget: u64, gas_objects: Vec<Input>) {
    ptb.set_sender(sender_address);
    ptb.set_gas_price(gas_price);
    ptb.set_gas_budget(gas_budget);
    ptb.add_gas_objects(gas_objects);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sui_types::{Transaction, TransactionKind};

    #[test]
    fn test_decode_transaction() {
        let tx = "AAAPAAhkx5NBAAAAAAAIKUO8sgMAAAAAAQAAAQAAAQAACGTHk0EAAAAAAQFexM/GvrUlJRacMqd+FsKIt7/Lm4mCielL8xCFcLPvpBbjZwAAAAAAAQEB2qRikmMsPE2PMfI+oPmzaij/NnfpaEmA5EOEA6Z6PY8uBRgAAAAAAAABAYBJ0AkRYmmsBO4UIGt6/YtktYASefhUAe5LOXefgJE0zicvAAAAAAABAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAEB8ZTZsbytly5Fp91n3Umz7h4zV6AKUIUMUs1Ru0UOE7QXwmUAAAAAAAABASjkmd/16GSi6v5HYmmk9QNfHBbzONp74YsQNJmr8nHO7fIyAAAAAAABAQHwxA1nsHgADhgDIzTDMlxHueyfPZrkEovoINVGY9FOO+/yMgAAAAAAAQEBNdNbDlsXdZPYw6gBRiSFVy/DCGHmzpalWvbcRzBwknju8jIAAAAAAAAAIJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2BgIAAQEAAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFvYnRhaW5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAUCAAABAQABAgABAwABBAAA3BVyG6qCumSCLVhac0mhUI922UroDombBuSDacJXdQ4Ic3dhcF9jYXANaW5pdGlhdGVfcGF0aAEHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQACAgEAAQUAAB7GqMWsC4uXwofNNLn8apS1OgfJMKhQWVJnncjUs3gKBnJvdXRlchBzd2FwX2JfdG9fYV9ieV9iAwcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgNzdWkDU1VJAAfkI5zZUfbFPZxB4lJw2A0x+SWtFlXlultUOEPUpml17gRTVUlQBFNVSVAABwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA3N1aQNTVUkABgEGAAIBAAEHAAEIAAICAAEJAADcFXIbqoK6ZIItWFpzSaFQj3bZSugOiZsG5INpwld1Dghzd2FwX2NhcBFyZXR1cm5fcm91dGVyX2NhcAIHAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIDc3VpA1NVSQAH5COc2VH2xT2cQeJScNgNMfklrRZV5bpbVDhD1KZpde4EU1VJUARTVUlQAAYCAQACAwABCgABCwABDAABDQABAQIDAAEOAJP2W4wWwmM0O79mz5+O72nLHbyS0T8MMxsNyut2tKq2AQAX1Cs2B1S8591qpdZjDUOB/CBDy2V8/6tqhBbwbdyxj734BAAAAAAg6yrtiW5R0TC68GDMmZye6U+KDjfZlq21n3bztRGzXjuT9luMFsJjNDu/Zs+fju9pyx28ktE/DDMbDcrrdrSqtu4CAAAAAAAA3P9fAAAAAAAA";
        let tx_data: Transaction = decode_transaction(tx).unwrap();

        assert_eq!(tx_data.sender.to_string(), "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6");
        match tx_data.kind {
            TransactionKind::ProgrammableTransaction(programmable) => {
                assert_eq!(programmable.commands.len(), 6);
            }
            _ => panic!("wrong kind"),
        }

        let output = validate_and_hash(tx).unwrap();
        assert_eq!(hex::encode(output.hash), "883f6f54145fdaf357e3d404a8353b1f6eda265bc2b28ec8178631e092c24e3b");
    }
}
