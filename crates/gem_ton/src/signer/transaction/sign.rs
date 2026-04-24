use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use num_bigint::BigUint;
use primitives::{FeeOption, SignerError, SignerInput};

use super::{
    message::{InternalMessage, build_internal_message},
    request::{JettonTransferRequest, TransferRequest},
};
use crate::address::Address;
use crate::signer::cells::{BagOfCells, CellBuilder};
use crate::signer::signer::TonSigner;

const STATE_INIT_EXPIRE_AT: u32 = u32::MAX;
const EXTERNAL_EXPIRE_WINDOW_SECS: u64 = 600;

impl TonSigner {
    pub fn sign_transfer(&self, input: &SignerInput, expire_at: Option<u32>) -> Result<String, SignerError> {
        let request = TransferRequest::new_transfer(&input.destination_address, &input.value, input.is_max_value, input.memo.clone())?;
        self.sign_requests(vec![request], input.metadata.get_sequence()?, expire_at)
    }

    pub fn sign_token_transfer(&self, input: &SignerInput, expire_at: Option<u32>) -> Result<String, SignerError> {
        let sender_token_address = input
            .metadata
            .get_sender_token_address()?
            .ok_or_else(|| SignerError::invalid_input("missing sender token address"))?;

        let jetton = JettonTransferRequest {
            query_id: 0,
            value: BigUint::from_str(&input.value)?,
            destination: Address::parse(&input.destination_address)?,
            response_address: Address::parse(&input.sender_address)?,
            custom_payload: None,
            forward_ton_amount: BigUint::from(1u8),
            comment: input.memo.clone(),
        };
        let request = TransferRequest::new_jetton_transfer(&sender_token_address, token_account_creation_fee(input)?, jetton)?;
        self.sign_requests(vec![request], input.metadata.get_sequence()?, expire_at)
    }

    pub fn sign_swap(&self, input: &SignerInput, expire_at: Option<u32>) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data()?;
        let request = TransferRequest::new_with_payload(
            &swap_data.data.to,
            &swap_data.data.value,
            input.memo.clone(),
            Some(BagOfCells::parse_base64_root(&swap_data.data.data)?),
            true,
            None,
        )?;
        Ok(vec![self.sign_requests(vec![request], input.metadata.get_sequence()?, expire_at)?])
    }

    pub(crate) fn sign_requests(&self, requests: Vec<TransferRequest>, sequence: u64, expire_at: Option<u32>) -> Result<String, SignerError> {
        let sequence = u32::try_from(sequence).map_err(|_| SignerError::invalid_input("TON sequence does not fit in u32"))?;
        let expire_at = resolve_expire_at(sequence, expire_at)?;

        let internal_messages: Vec<InternalMessage> = requests.iter().map(build_internal_message).collect::<Result<_, _>>()?;
        let external_body = self.wallet().build_external_body(expire_at, sequence, &internal_messages)?;
        let signature = self.sign(&external_body.hash);
        let mut body_builder = CellBuilder::new();
        body_builder.store_slice(&signature)?.store_cell(&external_body)?;
        let signed_transaction = self.wallet().build_transaction(sequence == 0, body_builder.build()?)?;

        BagOfCells::from_root(signed_transaction).to_base64(true)
    }
}

fn token_account_creation_fee(input: &SignerInput) -> Result<BigUint, SignerError> {
    let Some(value) = input.fee.options.get(&FeeOption::TokenAccountCreation) else {
        return Ok(BigUint::ZERO);
    };
    value.to_biguint().ok_or_else(|| SignerError::invalid_input("invalid TON amount"))
}

fn resolve_expire_at(sequence: u32, expire_at: Option<u32>) -> Result<u32, SignerError> {
    match (sequence, expire_at) {
        (0, _) => Ok(STATE_INIT_EXPIRE_AT),
        (_, Some(value)) => Ok(value),
        (_, None) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(SignerError::from_display)?.as_secs();
            u32::try_from(now + EXTERNAL_EXPIRE_WINDOW_SECS).map_err(|_| SignerError::invalid_input("TON expire time does not fit in u32"))
        }
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{
        Address as AddressTrait, Asset, AssetId, AssetType, Chain, SignerInput, TransactionInputType, TransactionLoadMetadata, swap::SwapData,
    };

    use super::super::{
        message::{DEFAULT_SEND_MODE, build_internal_message},
        request::{JettonTransferRequest, TransferPayload, TransferRequest},
        wallet::WalletV4R2,
    };
    use crate::address::Address;
    use crate::signer::TonSigner;
    use crate::signer::testkit::mock_cell;

    const TEST_TON_PRIVATE_KEY: &str = "c7702dadcd00d470df27dee0ddd97fbcf9deba52b60f7dd2b296ff42bb1fcad6";
    const TRUST_WALLET_PRIVATE_KEY: &str = "63474e5fe9511f1526a50567ce142befc343e71a49b865ac3908f58667319cb8";
    const SENDER_TOKEN_ADDRESS: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";
    const JETTON_ASSET_ADDRESS: &str = "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs";

    fn test_signer() -> TonSigner {
        let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
        TonSigner::new(&private_key).unwrap()
    }

    #[test]
    fn test_sign_matches_vectors() {
        let signer = test_signer();
        let address = signer.address().encode();

        // Native transfer — Android parity vector.
        let native_input = SignerInput::mock_with_input_type(
            TransactionInputType::Transfer(Asset::from_chain(Chain::Ton)),
            &address,
            &address,
            "10000",
            TransactionLoadMetadata::mock_ton(1),
        );
        assert_eq!(
            signer.sign_transfer(&native_input, Some(1_000_000_000)).unwrap(),
            "te6cckEBBAEArgABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGcOZ5W/jkCqNSj9wrP3isRN8k2PsJvAS1Rc7K+ABk/VgsvD4MSlcEFpS56SGhkmC7pSYwJM1Ocd7iIVUCY1DeFAimpoxc7msoAAAAAAQADAgFkQgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85BE4gAAAAAAAAAAAAAAAAAEDAABvNxKJ"
        );

        // Jetton transfer — Android parity vector.
        let asset = Asset::new(AssetId::from_token(Chain::Ton, JETTON_ASSET_ADDRESS), String::new(), String::new(), 8, AssetType::TOKEN);
        let jetton_input = SignerInput::mock_with_input_type(
            TransactionInputType::Transfer(asset),
            &address,
            &address,
            "10000",
            TransactionLoadMetadata::mock_ton_jetton(1, SENDER_TOKEN_ADDRESS),
        );
        assert_eq!(
            signer.sign_token_transfer(&jetton_input, Some(1_000_000_000)).unwrap(),
            "te6cckEBBAEA/wABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGcbaO6bjRLkbewbUrj8cYUocJI7vJDeXH4uoZqtTZzf5CRVBRw8rjMKMNg4MEafTwywe6wo2+BhefXkhOtdEakCympoxc7msoAAAAAAQADAgFgYgASwA6bnRklOr1y4MxDEh82TpZnlC7Kl8tiVkz/uVEgGgAAAAAAAAAAAAAAAAABAwCmD4p+pQAAAAAAAAAAInEIAZBdcOu3ASxtHtA+49Mtkoc35Qntia5QI0uKQvFB4PORADILrh124CWNo9oH3HplslDm/KE9sTXKBGlxSF4oPB5yAgKLD74O"
        );

        // Deploy — TrustWallet wallet-core parity vector:
        // https://github.com/trustwallet/wallet-core/blob/master/rust/tw_tests/tests/chains/ton/ton_sign.rs
        let deploy_private_key = hex::decode(TRUST_WALLET_PRIVATE_KEY).unwrap();
        let deploy_signer = TonSigner::new(&deploy_private_key).unwrap();
        let deploy_request = TransferRequest {
            destination: Address::parse("EQDYW_1eScJVxtitoBRksvoV9cCYo4uKGWLVNIHB1JqRR3n0").unwrap(),
            value: BigUint::from(10u8),
            mode: DEFAULT_SEND_MODE,
            bounceable: true,
            comment: None,
            payload: None,
            state_init: None,
        };
        assert_eq!(
            deploy_signer.sign_requests(vec![deploy_request], 0, Some(1_671_135_440)).unwrap(),
            "te6cckECGgEAA7IAAkWIAM33x4uAd+uQTyXyCZPxflESlNVHpCeoOECtNsqVW9tmHgECAgE0AwQBnOfG8YGGhFeE+iDE1jxCYeWKElbGDm3oqm2pwAhmVWSzWv5n6vVq8JY0J6p4sL+hqJU3iYPH8TX5mGLfcbbmtwgpqaMX/////wAAAAAAAwUBFP8A9KQT9LzyyAsGAFEAAAAAKamjF/Qsd/kxvqIOxdAVBzEna7suKGCUdmEkWyMZ74Ez7o1BQAFiYgBsLf6vJOEq42xW0AoyWX0K+uBMUcXFDLFqmkDg6k1Io4hQAAAAAAAAAAAAAAAAAQcCASAICQAAAgFICgsE+PKDCNcYINMf0x/THwL4I7vyZO1E0NMf0x/T//QE0VFDuvKhUVG68qIF+QFUEGT5EPKj+AAkpMjLH1JAyx9SMMv/UhD0AMntVPgPAdMHIcAAn2xRkyDXSpbTB9QC+wDoMOAhwAHjACHAAuMAAcADkTDjDQOkyMsfEssfy/8MDQ4PAubQAdDTAyFxsJJfBOAi10nBIJJfBOAC0x8hghBwbHVnvSKCEGRzdHK9sJJfBeAD+kAwIPpEAcjKB8v/ydDtRNCBAUDXIfQEMFyBAQj0Cm+hMbOSXwfgBdM/yCWCEHBsdWe6kjgw4w0DghBkc3RyupJfBuMNEBECASASEwBu0gf6ANTUIvkABcjKBxXL/8nQd3SAGMjLBcsCIs8WUAX6AhTLaxLMzMlz+wDIQBSBAQj0UfKnAgBwgQEI1xj6ANM/yFQgR4EBCPRR8qeCEG5vdGVwdIAYyMsFywJQBs8WUAT6AhTLahLLH8s/yXP7AAIAbIEBCNcY+gDTPzBSJIEBCPRZ8qeCEGRzdHJwdIAYyMsFywJQBc8WUAP6AhPLassfEss/yXP7AAAK9ADJ7VQAeAH6APQEMPgnbyIwUAqhIb7y4FCCEHBsdWeDHrFwgBhQBMsFJs8WWPoCGfQAy2kXyx9SYMs/IMmAQPsABgCKUASBAQj0WTDtRNCBAUDXIMgBzxb0AMntVAFysI4jghBkc3Rygx6xcIAYUAXLBVADzxYj+gITy2rLH8s/yYBA+wCSXwPiAgEgFBUAWb0kK29qJoQICga5D6AhhHDUCAhHpJN9KZEM5pA+n/mDeBKAG3gQFImHFZ8xhAIBWBYXABG4yX7UTQ1wsfgAPbKd+1E0IEBQNch9AQwAsjKB8v/ydABgQEI9ApvoTGACASAYGQAZrc52omhAIGuQ64X/wAAZrx32omhAEGuQ64WPwJiaP4Q="
        );
    }

    #[test]
    fn test_sign_swap_uses_custom_payload_transfer() {
        let signer = test_signer();
        let mut swap_data = SwapData::mock_with_provider(primitives::SwapProvider::StonfiV2);
        swap_data.data.to = SENDER_TOKEN_ADDRESS.to_string();
        swap_data.data.value = "241000000".to_string();
        swap_data.data.data = mock_cell();
        swap_data.data.gas_limit = None;
        let input = SignerInput::mock_with_input_type(
            TransactionInputType::Swap(Asset::from_chain(Chain::Ton), Asset::from_chain(Chain::Ton), swap_data),
            "",
            "",
            "0",
            TransactionLoadMetadata::mock_ton(1),
        );

        let signed = signer.sign_swap(&input, Some(1_000_000_000)).unwrap();
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
}
