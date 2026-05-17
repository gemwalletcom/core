use num_traits::ToPrimitive;
use number_formatter::BigNumberFormatter;
use primitives::{ChainSigner, SignerError, SignerInput, swap::SwapQuoteData};

use crate::address::XrpAddress;
use crate::signer::amount::XrpAmount;
use crate::signer::transaction::{XrpPaymentMemo, XrpTransaction, XrpTransactionParams};

const LEDGER_SEQUENCE_OFFSET: u64 = 12;
const TRUST_LINE_LIMIT: &str = "690000000000";

#[derive(Default)]
pub struct XrpChainSigner;

impl ChainSigner for XrpChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = XrpAmount::native(&input.value)?;

        sign_payment(input, private_key, amount, &input.destination_address, payment_memo(input.get_memo())?)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = token_amount(input, &input.value)?;
        sign_payment(input, private_key, amount, &input.destination_address, token_memo(input.get_memo())?)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let amount = XrpAmount::native(&swap.data.value)?;
        Ok(vec![sign_payment(input, private_key, amount, &swap.data.to, swap_memo(&swap.data))?])
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let amount = token_amount(input, TRUST_LINE_LIMIT)?;
        XrpTransaction::new_trust_set(params(input, private_key)?, amount).sign(private_key)
    }
}

fn sign_payment(input: &SignerInput, private_key: &[u8], amount: XrpAmount, destination: &str, memo: XrpPaymentMemo) -> Result<String, SignerError> {
    XrpTransaction::new_payment(params(input, private_key)?, amount, destination, memo)?.sign(private_key)
}

fn params(input: &SignerInput, private_key: &[u8]) -> Result<XrpTransactionParams, SignerError> {
    let block_number = input.metadata.get_block_number().map_err(SignerError::from_display)?;
    let last_ledger_sequence = block_number
        .checked_add(LEDGER_SEQUENCE_OFFSET)
        .ok_or_else(|| SignerError::invalid_input("XRP last ledger sequence overflow"))?;

    Ok(XrpTransactionParams {
        account: XrpAddress::parse(&input.sender_address)?,
        fee: input.fee.fee.to_u64().ok_or_else(|| SignerError::invalid_input("invalid XRP fee"))?,
        sequence: to_u32(input.metadata.get_sequence().map_err(SignerError::from_display)?, "XRP sequence")?,
        last_ledger_sequence: to_u32(last_ledger_sequence, "XRP last ledger sequence")?,
        signing_pub_key: ::signer::secp256k1_public_key(private_key)?,
    })
}

fn token_amount(input: &SignerInput, value: &str) -> Result<XrpAmount, SignerError> {
    let asset = input.input_type.get_asset();
    let value = BigNumberFormatter::value(value, asset.decimals).map_err(SignerError::from_display)?;
    XrpAmount::issued(&value, &asset.symbol, asset.id.get_token_id()?)
}

fn payment_memo(memo: Option<&str>) -> Result<XrpPaymentMemo, SignerError> {
    let Some(memo) = memo else {
        return Ok(XrpPaymentMemo::None);
    };

    match memo.parse::<u64>() {
        Ok(0) => Ok(XrpPaymentMemo::None),
        Ok(value) => Ok(XrpPaymentMemo::DestinationTag(to_u32(value, "XRP destination tag")?)),
        Err(_) => Ok(XrpPaymentMemo::Memo(memo_bytes(memo))),
    }
}

fn token_memo(memo: Option<&str>) -> Result<XrpPaymentMemo, SignerError> {
    let Some(memo) = memo else {
        return Ok(XrpPaymentMemo::None);
    };

    match memo.parse::<u64>() {
        Ok(0) | Err(_) => Ok(XrpPaymentMemo::None),
        Ok(value) => Ok(XrpPaymentMemo::DestinationTag(to_u32(value, "XRP destination tag")?)),
    }
}

fn swap_memo(data: &SwapQuoteData) -> XrpPaymentMemo {
    if let Some(memo) = data.memo.as_deref().filter(|memo| !memo.is_empty()) {
        return XrpPaymentMemo::Memo(memo.as_bytes().to_vec());
    }
    if data.data.is_empty() {
        return XrpPaymentMemo::None;
    }
    XrpPaymentMemo::Memo(data.data.as_bytes().to_vec())
}

fn memo_bytes(memo: &str) -> Vec<u8> {
    memo.strip_prefix("0x").unwrap_or(memo).as_bytes().to_vec()
}

fn to_u32(value: u64, name: &'static str) -> Result<u32, SignerError> {
    value.try_into().map_err(|_| SignerError::invalid_input(format!("{name} does not fit u32")))
}

#[cfg(test)]
mod tests {
    use primitives::{
        AccountDataType, Asset, AssetId, AssetType, Chain, GasPriceType, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata,
        swap::{SwapData, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType},
    };

    use super::*;

    fn metadata(sequence: u64, block_number: u64) -> TransactionLoadMetadata {
        TransactionLoadMetadata::Xrp { sequence, block_number }
    }

    fn signer_input(load: TransactionLoadInput, fee: u64) -> SignerInput {
        SignerInput::new(load, TransactionFee::new_from_fee(fee.into()))
    }

    fn transfer_input(asset: Asset, sender: &str, destination: &str, value: &str, fee: u64, sequence: u64, block_number: u64, memo: Option<&str>) -> SignerInput {
        signer_input(
            TransactionLoadInput::mock_transfer(asset, sender, destination, value, fee, memo, metadata(sequence, block_number)),
            fee,
        )
    }

    fn input_with_type(
        input_type: TransactionInputType,
        sender: &str,
        destination: &str,
        value: &str,
        fee: u64,
        sequence: u64,
        block_number: u64,
        memo: Option<&str>,
    ) -> SignerInput {
        signer_input(
            TransactionLoadInput {
                sender_address: sender.to_string(),
                destination_address: destination.to_string(),
                value: value.to_string(),
                gas_price: GasPriceType::regular(fee),
                memo: memo.map(str::to_string),
                metadata: metadata(sequence, block_number),
                ..TransactionLoadInput::mock_with_input_type(input_type)
            },
            fee,
        )
    }

    fn token(symbol: &str, issuer: &str) -> Asset {
        Asset::new(AssetId::from_token(Chain::Xrp, issuer), symbol.to_string(), symbol.to_string(), 15, AssetType::TOKEN)
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/62ef27c56c6769b3aec3c5167d925dc085646a5c/tests/chains/XRP/TWAnySignerTests.cpp#L17-L33
    #[test]
    fn test_sign_transfer_matches_wallet_core() {
        let private_key = hex::decode("a5576c0f63da10e584568c8d134569ff44017b0a249eb70657127ae04f38cc77").unwrap();
        let input = transfer_input(
            Asset::from_chain(Chain::Xrp),
            "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq",
            "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF",
            "10",
            10,
            32_268_248,
            32_268_257,
            None,
        );

        assert_eq!(
            XrpChainSigner.sign_transfer(&input, &private_key).unwrap(),
            "12000022000000002401ec5fd8201b01ec5fed61400000000000000a68400000000000000a732103d13e1152965a51a4a9fd9a8b4ea3dd82a4eba6b25fcad5f460a2342bb650333f74463044022037d32835c9394f39b2cfd4eaf5b0a80e0db397ace06630fa2b099ff73e425dbc02205288f780330b7a88a1980fa83c647b5908502ad7de9a44500c08f0750b0d9e8481144c55f5a78067206507580be7bb2686c8460adff983148132e4e20aecf29090ac428a9c43f230a829220d"
        );
    }

    // Source vector:
    // https://github.com/trustwallet/wallet-core/blob/62ef27c56c6769b3aec3c5167d925dc085646a5c/tests/chains/XRP/TWAnySignerTests.cpp#L134-L152
    #[test]
    fn test_sign_token_transfer_matches_wallet_core_custom_currency() {
        let private_key = hex::decode("574e99f7946cfa2a6ca9368ca72fd37e42583cddb9ecc746aa4cb194ef4b2480").unwrap();
        let issuer = "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De";
        let input = transfer_input(
            token("RLUSD", issuer),
            "rDgEGKXWkHHr1HYq2ETnNAs9MdV4R8Gyt",
            "r4oPb529jpRA1tVTDARmBuZPYB2CJjKFac",
            "1000000000000000",
            12,
            93_674_951,
            187_349_938,
            None,
        );

        assert_eq!(
            XrpChainSigner.sign_token_transfer(&input, &private_key).unwrap(),
            "12000022000000002405955dc7201b0b2abbbe61d4838d7ea4c68000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e68400000000000000c7321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d744630440220552e90f417c2cabe39368bb45cf7495ba6ebe395f259a6509c9f3a7296e76a0d02201b37dae0c4c77fa70a451cd4a61c10575c8b052c282c082a32c229e7624a05e381140265c09d122fab2a261a80ee59f1f4cd8fba8cf88314ef20a3d93b00cc729eec11a3058d3d1feb4465e0"
        );
    }

    #[test]
    fn test_destination_tag_and_memo() {
        assert_eq!(payment_memo(Some("123")).unwrap(), XrpPaymentMemo::DestinationTag(123));
        assert_eq!(payment_memo(Some("memo")).unwrap(), XrpPaymentMemo::Memo(b"memo".to_vec()));
        assert_eq!(payment_memo(Some("0xhello")).unwrap(), XrpPaymentMemo::Memo(b"hello".to_vec()));
        assert_eq!(payment_memo(Some("0")).unwrap(), XrpPaymentMemo::None);
        assert_eq!(payment_memo(None).unwrap(), XrpPaymentMemo::None);
        assert_eq!(token_memo(Some("123")).unwrap(), XrpPaymentMemo::DestinationTag(123));
        assert_eq!(token_memo(Some("0")).unwrap(), XrpPaymentMemo::None);
        assert_eq!(token_memo(Some("memo")).unwrap(), XrpPaymentMemo::None);
    }

    #[test]
    fn test_swap_memo_is_always_memo_data() {
        let data = SwapQuoteData {
            to: "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF".to_string(),
            data_type: SwapQuoteDataType::Transfer,
            value: "10".to_string(),
            data: "fallback".to_string(),
            memo: Some("123".to_string()),
            approval: None,
            gas_limit: None,
        };

        assert_eq!(swap_memo(&data), XrpPaymentMemo::Memo(b"123".to_vec()));
    }

    #[test]
    fn test_sign_swap_uses_payload_not_provider() {
        let private_key = hex::decode("a5576c0f63da10e584568c8d134569ff44017b0a249eb70657127ae04f38cc77").unwrap();
        let input = input_with_type(
            TransactionInputType::Swap(
                Asset::from_chain(Chain::Xrp),
                Asset::from_chain(Chain::Xrp),
                SwapData {
                    quote: SwapQuote {
                        from_address: "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq".to_string(),
                        from_value: "10".to_string(),
                        to_address: "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF".to_string(),
                        to_value: "1".to_string(),
                        provider_data: SwapProviderData {
                            provider: SwapProvider::Okx,
                            name: "OKX".to_string(),
                            protocol_name: "okx".to_string(),
                        },
                        slippage_bps: 50,
                        eta_in_seconds: None,
                        use_max_amount: None,
                    },
                    data: SwapQuoteData {
                        to: "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF".to_string(),
                        data_type: SwapQuoteDataType::Transfer,
                        value: "10".to_string(),
                        data: "swap:memo".to_string(),
                        memo: None,
                        approval: None,
                        gas_limit: None,
                    },
                },
            ),
            "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq",
            "",
            "999",
            10,
            32_268_248,
            32_268_257,
            None,
        );

        let signed = XrpChainSigner.sign_swap(&input, &private_key).unwrap();
        assert_eq!(signed.len(), 1);
        assert_eq!(
            signed[0],
            "12000022000000002401ec5fd8201b01ec5fed61400000000000000a68400000000000000a732103d13e1152965a51a4a9fd9a8b4ea3dd82a4eba6b25fcad5f460a2342bb650333f74473045022100d18b758a360fc0a4d013b095014410b4fbf0e97a265c10d01d85e86ff35e009a02203c7e860711fd18ca486d211c07669638cbfa39b5f01ccc430e12fdecfa11399281144c55f5a78067206507580be7bb2686c8460adff983148132e4e20aecf29090ac428a9c43f230a829220df9ea7d09737761703a6d656d6fe1f1"
        );
    }

    #[test]
    fn test_account_action_signs_trust_set() {
        let private_key = hex::decode("574e99f7946cfa2a6ca9368ca72fd37e42583cddb9ecc746aa4cb194ef4b2480").unwrap();
        let input = input_with_type(
            TransactionInputType::Account(token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"), AccountDataType::Activate),
            "rDgEGKXWkHHr1HYq2ETnNAs9MdV4R8Gyt",
            "",
            "0",
            500,
            93_674_950,
            187_349_938,
            None,
        );

        let signed = XrpChainSigner.sign_account_action(&input, &private_key).unwrap();
        assert_eq!(
            signed,
            "12001422000000002405955dc6201b0b2abbbe63d398838370f34000524c555344000000000000000000000000000000e5e961c6a025c9404aa7b662dd1df975be75d13e6840000000000001f47321039c77e9329017ced5f8673ebafcd29687a1fff181140c030062fa77865688fc5d74473045022100d807b19bc7636d2a4b92f3b1c27897f6076a0f808abf4a403188b50c6e4205fc02202c50debc5aed8c8e193dd68122502b75333557378e9537a88c19233e52870a7781140265c09d122fab2a261a80ee59f1f4cd8fba8cf8"
        );
    }
}
