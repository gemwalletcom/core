use bitcoin::{
    PublicKey,
    secp256k1::{PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey},
};
use num_bigint::BigInt;
use primitives::{
    Asset, BitcoinChain, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO,
    swap::{SwapData, SwapProviderData, SwapQuote, SwapQuoteData},
};

use crate::{
    signer::address::{UnlockingScript, public_key_hash, script_for_public_key_hash},
    testkit::address_mock::address_for_hash,
};

pub use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;

pub const TEST_UTXO_TXID: &str = "0000000000000000000000000000000000000000000000000000000000000001";
pub const TEST_ZCASH_BRANCH_ID: &str = "4dec4df0";

pub fn public_key() -> PublicKey {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&TEST_PRIVATE_KEY).unwrap();
    PublicKey::new(Secp256k1PublicKey::from_secret_key(&secp, &secret_key))
}

pub fn sender_address(chain: BitcoinChain) -> String {
    let public_key = public_key();
    address_for_hash(chain, public_key_hash(&public_key.to_bytes()))
}

pub fn destination_address(chain: BitcoinChain) -> String {
    let hash = match chain {
        BitcoinChain::Bitcoin => public_key_hash(&public_key().to_bytes()),
        BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => [2u8; 20],
        BitcoinChain::Zcash => [3u8; 20],
    };
    address_for_hash(chain, hash)
}

pub fn transfer_input(chain: BitcoinChain) -> SignerInput {
    let sender_address = sender_address(chain);
    let destination_address = destination_address(chain);
    transfer_input_with_utxos(chain, &sender_address, &destination_address, "10000", vec![utxo_with_address(&sender_address)])
}

pub fn transfer_input_with_utxos(chain: BitcoinChain, sender_address: &str, destination_address: &str, value: &str, utxos: Vec<UTXO>) -> SignerInput {
    let metadata = match chain {
        BitcoinChain::Zcash => TransactionLoadMetadata::Zcash {
            branch_id: TEST_ZCASH_BRANCH_ID.to_string(),
            utxos,
        },
        _ => TransactionLoadMetadata::Bitcoin { utxos },
    };

    SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(chain.get_chain())),
            sender_address: sender_address.to_string(),
            destination_address: destination_address.to_string(),
            value: value.to_string(),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata,
        },
        TransactionFee::new_from_fee(BigInt::from(1u64)),
    )
}

fn p2wpkh_address() -> String {
    let hash = public_key_hash(&public_key().to_bytes());
    let script_pubkey = script_for_public_key_hash(UnlockingScript::P2wpkh, hash);
    bitcoin::Address::from_script(&script_pubkey, bitcoin::Network::Bitcoin).unwrap().to_string()
}

pub fn p2wpkh_transfer_input() -> SignerInput {
    let address = p2wpkh_address();
    transfer_input_with_utxos(BitcoinChain::Bitcoin, &address, &address, "10000", vec![utxo_with(TEST_UTXO_TXID, 0, "50000", &address)])
}

pub fn funded_transfer_input(chain: BitcoinChain) -> SignerInput {
    let mut input = transfer_input(chain);
    match &mut input.input.metadata {
        TransactionLoadMetadata::Bitcoin { utxos } | TransactionLoadMetadata::Zcash { utxos, .. } => {
            utxos[0].value = "100000000".to_string();
        }
        _ => {}
    }
    input
}

pub fn transfer_swap_input(chain: BitcoinChain, memo: &str) -> SignerInput {
    swap_input(chain, SwapProvider::Thorchain, Some(false), |destination_address, value| {
        SwapQuoteData::new_tranfer(destination_address, value, Some(memo.to_string()))
    })
}

pub fn contract_swap_input(chain: BitcoinChain, nulldata_hex: &str, use_max_amount: bool) -> SignerInput {
    contract_swap_input_with_provider(chain, nulldata_hex, use_max_amount, SwapProvider::Chainflip)
}

pub fn contract_swap_input_with_provider(chain: BitcoinChain, nulldata_hex: &str, use_max_amount: bool, provider: SwapProvider) -> SignerInput {
    swap_input(chain, provider, Some(use_max_amount), |destination_address, value| {
        SwapQuoteData::new_contract(destination_address, value, nulldata_hex.to_string(), None, None)
    })
}

fn swap_input(chain: BitcoinChain, provider: SwapProvider, use_max_amount: Option<bool>, quote_data: impl FnOnce(String, String) -> SwapQuoteData) -> SignerInput {
    let mut input = funded_transfer_input(chain);
    let sender_address = input.sender_address.clone();
    let destination_address = input.destination_address.clone();
    let value = input.value.clone();

    input.input.input_type = TransactionInputType::Swap(
        Asset::from_chain(chain.get_chain()),
        Asset::from_chain(BitcoinChain::Bitcoin.get_chain()),
        SwapData {
            quote: SwapQuote {
                from_address: sender_address,
                from_value: value.clone(),
                min_from_value: None,
                to_address: destination_address.clone(),
                to_value: value.clone(),
                provider_data: SwapProviderData {
                    provider,
                    name: provider.name().to_string(),
                    protocol_name: provider.protocol_name().to_string(),
                },
                slippage_bps: 50,
                eta_in_seconds: None,
                use_max_amount,
            },
            data: quote_data(destination_address, value),
        },
    );
    input
}

pub fn p2wpkh_contract_swap_input(nulldata_hex: &str, use_max_amount: bool) -> SignerInput {
    let p2wpkh_sender = p2wpkh_address();
    let mut input = contract_swap_input(BitcoinChain::Bitcoin, nulldata_hex, use_max_amount);
    input.input.sender_address = p2wpkh_sender.clone();
    let TransactionLoadMetadata::Bitcoin { utxos } = &mut input.input.metadata else {
        unreachable!()
    };
    utxos[0].address = p2wpkh_sender.clone();
    let TransactionInputType::Swap(_, _, swap) = &mut input.input.input_type else {
        unreachable!()
    };
    swap.quote.from_address = p2wpkh_sender;
    input
}

pub(crate) fn utxo_with(transaction_id: &str, vout: i32, value: &str, address: &str) -> UTXO {
    UTXO {
        transaction_id: transaction_id.to_string(),
        vout,
        value: value.to_string(),
        address: address.to_string(),
    }
}

pub(crate) fn utxo_with_address(address: &str) -> UTXO {
    utxo_with(TEST_UTXO_TXID, 0, "50000", address)
}
