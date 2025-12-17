use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_network::TxSignerSync;
use alloy_network::eip2718::Encodable2718;
use alloy_signer_local::PrivateKeySigner;
use std::error::Error;

pub fn sign_eip1559_tx(tx: &TxEip1559, private_key: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let signer = PrivateKeySigner::from_slice(private_key)?;
    let mut tx = tx.clone();
    let signature = signer.sign_transaction_sync(&mut tx)?;
    let signed = tx.into_signed(signature);
    Ok(signed.encoded_2718())
}
