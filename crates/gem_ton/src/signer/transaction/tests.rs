use num_bigint::BigUint;
use primitives::{
    Asset, AssetId, AssetType, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra,
    TransferDataOutputAction, TransferDataOutputType, WalletConnectionSessionAppMetadata,
    swap::{SwapData, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType},
};
use signer::Ed25519KeyPair;

use super::request::TransferRequest;
use super::signing::{DEFAULT_SEND_MODE, WalletV4R2, parse_address, sign_data, sign_requests, sign_swap, sign_token_transfer, sign_transfer};
use crate::signer::cells::{BagOfCells, CellBuilder};

const TEST_TON_PRIVATE_KEY: &str = "c7702dadcd00d470df27dee0ddd97fbcf9deba52b60f7dd2b296ff42bb1fcad6";
const TRUST_WALLET_PRIVATE_KEY: &str = "63474e5fe9511f1526a50567ce142befc343e71a49b865ac3908f58667319cb8";

fn mock_wc_metadata() -> WalletConnectionSessionAppMetadata {
    WalletConnectionSessionAppMetadata {
        name: "Test Dapp".to_string(),
        description: "Test Dapp".to_string(),
        url: "https://example.com".to_string(),
        icon: "https://example.com/icon.png".to_string(),
    }
}

fn sample_boc() -> String {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, 0x12345678).unwrap();
    BagOfCells::from_root(builder.build().unwrap()).to_base64(true).unwrap()
}

#[test]
fn test_sign_native_transfer_matches_android_vector() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let wallet = WalletV4R2::new(Ed25519KeyPair::from_private_key(&private_key).unwrap().public_key_bytes).unwrap();
    let address = wallet.address().to_base64_url();

    let input = SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Ton)),
            sender_address: address.clone(),
            destination_address: address,
            value: "10000".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                sender_token_address: None,
                recipient_token_address: None,
                sequence: 1,
            },
        },
        TransactionFee::default(),
    );

    let signed = sign_transfer(&input, &private_key, Some(1_000_000_000)).unwrap();
    assert_eq!(
        signed,
        "te6cckEBBAEArgABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGcOZ5W/jkCqNSj9wrP3isRN8k2PsJvAS1Rc7K+ABk/VgsvD4MSlcEFpS56SGhkmC7pSYwJM1Ocd7iIVUCY1DeFAimpoxc7msoAAAAAAQADAgFkQgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85BE4gAAAAAAAAAAAAAAAAAEDAABvNxKJ"
    );
}

#[test]
fn test_sign_jetton_transfer_matches_android_vector() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let wallet = WalletV4R2::new(Ed25519KeyPair::from_private_key(&private_key).unwrap().public_key_bytes).unwrap();
    let address = wallet.address().to_base64_url();

    let input = SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::new(
                AssetId::from_token(Chain::Ton, "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"),
                "".to_string(),
                "".to_string(),
                8,
                AssetType::TOKEN,
            )),
            sender_address: address.clone(),
            destination_address: address,
            value: "10000".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                sender_token_address: Some("EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9".to_string()),
                recipient_token_address: None,
                sequence: 1,
            },
        },
        TransactionFee::default(),
    );

    let signed = sign_token_transfer(&input, &private_key, Some(1_000_000_000)).unwrap();
    assert_eq!(
        signed,
        "te6cckEBBAEA/wABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGcbaO6bjRLkbewbUrj8cYUocJI7vJDeXH4uoZqtTZzf5CRVBRw8rjMKMNg4MEafTwywe6wo2+BhefXkhOtdEakCympoxc7msoAAAAAAQADAgFgYgASwA6bnRklOr1y4MxDEh82TpZnlC7Kl8tiVkz/uVEgGgAAAAAAAAAAAAAAAAABAwCmD4p+pQAAAAAAAAAAInEIAZBdcOu3ASxtHtA+49Mtkoc35Qntia5QI0uKQvFB4PORADILrh124CWNo9oH3HplslDm/KE9sTXKBGlxSF4oPB5yAgKLD74O"
    );
}

// Parity vector from wallet-core `test_ton_sign_transfer_and_deploy`:
// https://github.com/trustwallet/wallet-core/blob/master/rust/tw_tests/tests/chains/ton/ton_sign.rs
#[test]
fn test_sign_deploy_matches_trust_wallet_core_vector() {
    let private_key = hex::decode(TRUST_WALLET_PRIVATE_KEY).unwrap();
    let request = TransferRequest {
        destination: parse_address("EQDYW_1eScJVxtitoBRksvoV9cCYo4uKGWLVNIHB1JqRR3n0").unwrap(),
        value: BigUint::from(10u8),
        mode: DEFAULT_SEND_MODE,
        bounceable: true,
        comment: None,
        payload: None,
        state_init: None,
    };

    let signed = sign_requests(vec![request], 0, &private_key, Some(1_671_135_440)).unwrap();
    assert_eq!(
        signed,
        "te6cckECGgEAA7IAAkWIAM33x4uAd+uQTyXyCZPxflESlNVHpCeoOECtNsqVW9tmHgECAgE0AwQBnOfG8YGGhFeE+iDE1jxCYeWKElbGDm3oqm2pwAhmVWSzWv5n6vVq8JY0J6p4sL+hqJU3iYPH8TX5mGLfcbbmtwgpqaMX/////wAAAAAAAwUBFP8A9KQT9LzyyAsGAFEAAAAAKamjF/Qsd/kxvqIOxdAVBzEna7suKGCUdmEkWyMZ74Ez7o1BQAFiYgBsLf6vJOEq42xW0AoyWX0K+uBMUcXFDLFqmkDg6k1Io4hQAAAAAAAAAAAAAAAAAQcCASAICQAAAgFICgsE+PKDCNcYINMf0x/THwL4I7vyZO1E0NMf0x/T//QE0VFDuvKhUVG68qIF+QFUEGT5EPKj+AAkpMjLH1JAyx9SMMv/UhD0AMntVPgPAdMHIcAAn2xRkyDXSpbTB9QC+wDoMOAhwAHjACHAAuMAAcADkTDjDQOkyMsfEssfy/8MDQ4PAubQAdDTAyFxsJJfBOAi10nBIJJfBOAC0x8hghBwbHVnvSKCEGRzdHK9sJJfBeAD+kAwIPpEAcjKB8v/ydDtRNCBAUDXIfQEMFyBAQj0Cm+hMbOSXwfgBdM/yCWCEHBsdWe6kjgw4w0DghBkc3RyupJfBuMNEBECASASEwBu0gf6ANTUIvkABcjKBxXL/8nQd3SAGMjLBcsCIs8WUAX6AhTLaxLMzMlz+wDIQBSBAQj0UfKnAgBwgQEI1xj6ANM/yFQgR4EBCPRR8qeCEG5vdGVwdIAYyMsFywJQBs8WUAT6AhTLahLLH8s/yXP7AAIAbIEBCNcY+gDTPzBSJIEBCPRZ8qeCEGRzdHJwdIAYyMsFywJQBc8WUAP6AhPLassfEss/yXP7AAAK9ADJ7VQAeAH6APQEMPgnbyIwUAqhIb7y4FCCEHBsdWeDHrFwgBhQBMsFJs8WWPoCGfQAy2kXyx9SYMs/IMmAQPsABgCKUASBAQj0WTDtRNCBAUDXIMgBzxb0AMntVAFysI4jghBkc3Rygx6xcIAYUAXLBVADzxYj+gITy2rLH8s/yYBA+wCSXwPiAgEgFBUAWb0kK29qJoQICga5D6AhhHDUCAhHpJN9KZEM5pA+n/mDeBKAG3gQFImHFZ8xhAIBWBYXABG4yX7UTQ1wsfgAPbKd+1E0IEBQNch9AQwAsjKB8v/ydABgQEI9ApvoTGACASAYGQAZrc52omhAIGuQ64X/wAAZrx32omhAEGuQ64WPwJiaP4Q="
    );
}

#[test]
fn test_sign_data_supports_state_init() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let wallet = WalletV4R2::new(Ed25519KeyPair::from_private_key(&private_key).unwrap().public_key_bytes).unwrap();
    let address = wallet.address().to_base64_url();
    let payload_boc = sample_boc();
    let payload = format!(r#"[{{"address":"EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9","amount":"241000000","payload":"{payload_boc}","stateInit":"{payload_boc}"}}]"#);

    let input = SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Generic(
                Asset::from_chain(Chain::Ton),
                mock_wc_metadata(),
                TransferDataExtra {
                    to: "".to_string(),
                    gas_limit: None,
                    gas_price: None,
                    data: Some(payload.into_bytes()),
                    output_type: TransferDataOutputType::EncodedTransaction,
                    output_action: TransferDataOutputAction::Send,
                },
            ),
            sender_address: address,
            destination_address: "".to_string(),
            value: "0".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                sender_token_address: None,
                recipient_token_address: None,
                sequence: 1,
            },
        },
        TransactionFee::default(),
    );

    let signed = sign_data(&input, &private_key, Some(1_000_000_000)).unwrap();
    assert!(signed.starts_with("te6cc"));
}

#[test]
fn test_sign_swap_uses_custom_payload_transfer() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let payload_boc = sample_boc();
    let input = SignerInput::new(
        TransactionLoadInput {
            input_type: TransactionInputType::Swap(
                Asset::from_chain(Chain::Ton),
                Asset::from_chain(Chain::Ton),
                SwapData {
                    quote: SwapQuote {
                        from_address: "from".to_string(),
                        from_value: "1000".to_string(),
                        to_address: "to".to_string(),
                        to_value: "900".to_string(),
                        provider_data: SwapProviderData {
                            provider: primitives::SwapProvider::StonfiV2,
                            name: "STON.fi".to_string(),
                            protocol_name: "STON.fi".to_string(),
                        },
                        slippage_bps: 100,
                        eta_in_seconds: None,
                        use_max_amount: None,
                    },
                    data: SwapQuoteData {
                        to: "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9".to_string(),
                        data_type: SwapQuoteDataType::Contract,
                        value: "241000000".to_string(),
                        data: payload_boc,
                        memo: None,
                        approval: None,
                        gas_limit: None,
                    },
                },
            ),
            sender_address: "".to_string(),
            destination_address: "".to_string(),
            value: "0".to_string(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Ton {
                sender_token_address: None,
                recipient_token_address: None,
                sequence: 1,
            },
        },
        TransactionFee::default(),
    );

    let signed = sign_swap(&input, &private_key, Some(1_000_000_000)).unwrap();
    assert_eq!(signed.len(), 1);
    assert!(signed[0].starts_with("te6cc"));
}
