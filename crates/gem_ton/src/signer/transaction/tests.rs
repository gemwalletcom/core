use num_bigint::BigUint;
use primitives::{
    Address as AddressTrait, Asset, AssetId, AssetType, Chain, SignerInput, TransactionInputType, TransactionLoadMetadata, TransferDataExtra, WCTonMessage, WCTonSendTransaction,
    WalletConnectionSessionAppMetadata, swap::SwapData,
};
use signer::Ed25519KeyPair;

use super::{
    message::{DEFAULT_SEND_MODE, build_internal_message},
    request::{JettonTransferRequest, TransferPayload, TransferRequest},
    signing::{sign_data, sign_requests, sign_swap, sign_token_transfer, sign_transfer},
    wallet::WalletV4R2,
};
use crate::address::Address;
use crate::signer::cells::{BagOfCells, BitReader, CellBuilder};

const TEST_TON_PRIVATE_KEY: &str = "c7702dadcd00d470df27dee0ddd97fbcf9deba52b60f7dd2b296ff42bb1fcad6";
const TRUST_WALLET_PRIVATE_KEY: &str = "63474e5fe9511f1526a50567ce142befc343e71a49b865ac3908f58667319cb8";
const SENDER_TOKEN_ADDRESS: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";
const JETTON_ASSET_ADDRESS: &str = "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs";
const WC_TON_MESSAGE_FIXTURE: &str = include_str!("../../../testdata/wc_ton_message.json");

fn test_wallet_address() -> String {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let key_pair = Ed25519KeyPair::from_private_key(&private_key).unwrap();
    WalletV4R2::new(key_pair.public_key_bytes).unwrap().address().encode()
}

fn sample_boc() -> String {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, 0x12345678).unwrap();
    BagOfCells::from_root(builder.build().unwrap()).to_base64(true).unwrap()
}

fn wc_ton_payload(payload_boc: &str) -> Vec<u8> {
    WC_TON_MESSAGE_FIXTURE.trim().replace("{PAYLOAD}", payload_boc).into_bytes()
}

fn wc_ton_request(payload_boc: &str, valid_until: Option<u32>, from: Option<&str>) -> Vec<u8> {
    let messages: Vec<WCTonMessage> = serde_json::from_str(&WC_TON_MESSAGE_FIXTURE.trim().replace("{PAYLOAD}", payload_boc)).unwrap();
    serde_json::to_vec(&WCTonSendTransaction {
        valid_until,
        messages,
        r#from: from.map(|from| from.to_string()),
        network: None,
    })
    .unwrap()
}

fn signed_expire_at(signed: &str) -> u32 {
    let signed = BagOfCells::parse_base64(signed).unwrap();
    let root = signed.single_root().unwrap();
    let signed_body = root.references.last().unwrap();
    let mut reader = BitReader::from_bits(&signed_body.data, signed_body.bit_len).unwrap();
    reader.read_bytes(64).unwrap();
    let _wallet_id = reader.read_u32().unwrap();
    reader.read_u32().unwrap()
}

#[test]
fn test_sign_native_transfer_matches_android_vector() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let address = test_wallet_address();
    let input = SignerInput::mock_with_input_type(
        TransactionInputType::Transfer(Asset::from_chain(Chain::Ton)),
        &address,
        &address,
        "10000",
        TransactionLoadMetadata::mock_ton(1),
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
    let address = test_wallet_address();
    let asset = Asset::new(AssetId::from_token(Chain::Ton, JETTON_ASSET_ADDRESS), String::new(), String::new(), 8, AssetType::TOKEN);
    let input = SignerInput::mock_with_input_type(
        TransactionInputType::Transfer(asset),
        &address,
        &address,
        "10000",
        TransactionLoadMetadata::mock_ton_jetton(1, SENDER_TOKEN_ADDRESS),
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
        destination: Address::parse("EQDYW_1eScJVxtitoBRksvoV9cCYo4uKGWLVNIHB1JqRR3n0").unwrap(),
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
fn test_sign_data_supports_request_envelopes() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let address = test_wallet_address();
    let extra = TransferDataExtra::mock_encoded_transaction(wc_ton_request(&sample_boc(), Some(1_000_000_000), Some(&address)));
    let input = SignerInput::mock_with_input_type(
        TransactionInputType::Generic(Asset::from_chain(Chain::Ton), WalletConnectionSessionAppMetadata::mock(), extra),
        &address,
        "",
        "0",
        TransactionLoadMetadata::mock_ton(1),
    );

    let signed = sign_data(&input, &private_key, None).unwrap();
    assert!(signed.starts_with("te6cc"));
    assert_eq!(signed_expire_at(&signed), 1_000_000_000);

    let legacy = SignerInput::mock_with_input_type(
        TransactionInputType::Generic(
            Asset::from_chain(Chain::Ton),
            WalletConnectionSessionAppMetadata::mock(),
            TransferDataExtra::mock_encoded_transaction(wc_ton_payload(&sample_boc())),
        ),
        &address,
        "",
        "0",
        TransactionLoadMetadata::mock_ton(1),
    );

    let signed_legacy = sign_data(&legacy, &private_key, Some(1_000_000_000)).unwrap();
    assert!(signed_legacy.starts_with("te6cc"));

    let mismatched = SignerInput::mock_with_input_type(
        TransactionInputType::Generic(
            Asset::from_chain(Chain::Ton),
            WalletConnectionSessionAppMetadata::mock(),
            TransferDataExtra::mock_encoded_transaction(wc_ton_request(&sample_boc(), Some(1_000_000_000), Some(SENDER_TOKEN_ADDRESS))),
        ),
        &address,
        "",
        "0",
        TransactionLoadMetadata::mock_ton(1),
    );

    assert_eq!(
        sign_data(&mismatched, &private_key, None).unwrap_err().to_string(),
        "Invalid input: TON from does not match sender address"
    );
}

#[test]
fn test_sign_swap_uses_custom_payload_transfer() {
    let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
    let mut swap_data = SwapData::mock_with_provider(primitives::SwapProvider::StonfiV2);
    swap_data.data.to = SENDER_TOKEN_ADDRESS.to_string();
    swap_data.data.value = "241000000".to_string();
    swap_data.data.data = sample_boc();
    swap_data.data.gas_limit = None;
    let input = SignerInput::mock_with_input_type(
        TransactionInputType::Swap(Asset::from_chain(Chain::Ton), Asset::from_chain(Chain::Ton), swap_data),
        "",
        "",
        "0",
        TransactionLoadMetadata::mock_ton(1),
    );

    let signed = sign_swap(&input, &private_key, Some(1_000_000_000)).unwrap();
    assert_eq!(signed.len(), 1);
    assert!(signed[0].starts_with("te6cc"));
}

#[test]
fn test_long_comments_use_snake_cells() {
    let address = Address::parse(SENDER_TOKEN_ADDRESS).unwrap();
    let comment = "memo".repeat(80);

    let transfer = TransferRequest {
        destination: address,
        value: BigUint::from(10u8),
        mode: DEFAULT_SEND_MODE,
        bounceable: false,
        comment: Some(comment.clone()),
        payload: None,
        state_init: None,
    };
    let native_payload = build_internal_message(&transfer).unwrap().message.references.first().unwrap().clone();
    assert!(!native_payload.references.is_empty());

    let jetton = TransferRequest {
        destination: address,
        value: BigUint::ZERO,
        mode: DEFAULT_SEND_MODE,
        bounceable: true,
        comment: None,
        payload: Some(TransferPayload::Jetton(JettonTransferRequest {
            query_id: 0,
            value: BigUint::from(10u8),
            destination: address,
            response_address: address,
            custom_payload: None,
            forward_ton_amount: BigUint::from(1u8),
            comment: Some(comment),
        })),
        state_init: None,
    };
    let jetton_payload = build_internal_message(&jetton).unwrap().message.references.first().unwrap().clone();
    assert_eq!(jetton_payload.references.len(), 1);
    assert!(!jetton_payload.references[0].references.is_empty());
}
