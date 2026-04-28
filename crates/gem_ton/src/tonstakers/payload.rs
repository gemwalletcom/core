use std::str::FromStr;

use num_bigint::BigUint;
use primitives::{EarnType, SignerError};

use crate::{
    Address,
    signer::cells::{BagOfCells, CellBuilder},
};

// Placeholder id "GEM" + tagged timestamp; replace once Tonstakers issues an official partner_id.
const PARTNER_CODE: u64 = 0x47454D0069EEF127;
const STAKE_OPCODE: u32 = 0x47D54391;
const STAKE_FEE: u64 = 1_000_000_000;
const UNSTAKE_OPCODE: u32 = 0x595F07BC;
const UNSTAKE_FEE: u64 = 1_050_000_000;

pub(crate) fn attached_value(earn_type: &EarnType, value: &str) -> Result<BigUint, SignerError> {
    match earn_type {
        EarnType::Deposit(_) => Ok(BigUint::from_str(value)? + BigUint::from(STAKE_FEE)),
        EarnType::Withdraw(_) => Ok(BigUint::from(UNSTAKE_FEE)),
    }
}

pub fn build_stake_payload_base64() -> Result<String, SignerError> {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, STAKE_OPCODE)?.store_u64(64, 1)?.store_u64(64, PARTNER_CODE)?;
    BagOfCells::from_root(builder.build()?).to_base64(true)
}

pub fn build_unstake_payload_base64(owner: &Address, amount: &BigUint) -> Result<String, SignerError> {
    // Tonstakers unstake flags cell: wait_till_round_end=0, fill_or_kill=0.
    let mut flags = CellBuilder::new();
    flags.store_u8(2, 0)?;

    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, UNSTAKE_OPCODE)?
        .store_u64(64, 0)?
        .store_coins(amount)?
        .store_address(owner)?
        .store_bit(true)?
        .store_child(flags.build()?)?;
    BagOfCells::from_root(builder.build()?).to_base64(true)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, ContractCallData, EarnType, SignerInput, TransactionInputType, TransactionLoadMetadata, YieldProvider};

    use super::{build_stake_payload_base64, build_unstake_payload_base64};
    use crate::{Address, signer::TonSigner};

    const TEST_TON_PRIVATE_KEY: &str = "c7702dadcd00d470df27dee0ddd97fbcf9deba52b60f7dd2b296ff42bb1fcad6";
    const SENDER_TOKEN_ADDRESS: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";

    fn test_signer() -> TonSigner {
        let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
        TonSigner::new(&private_key).unwrap()
    }

    fn signer_input(earn_type: EarnType, call_data: String, value: &str) -> SignerInput {
        let input_type = TransactionInputType::Earn(
            Asset::from_chain(Chain::Ton),
            earn_type,
            ContractCallData::new(SENDER_TOKEN_ADDRESS.to_string(), call_data),
        );
        SignerInput::mock_with_input_type(input_type, "", "", value, TransactionLoadMetadata::mock_ton(1))
    }

    #[test]
    fn test_build_stake_payload_base64() {
        assert_eq!(build_stake_payload_base64().unwrap(), "te6cckEBAQEAFgAAKEfVQ5EAAAAAAAAAAUdFTQBp7vEnOxjgfw==");
    }

    #[test]
    fn test_build_unstake_payload_base64() {
        let owner = Address::parse("EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8").unwrap();
        assert_eq!(
            build_unstake_payload_base64(&owner, &BigUint::from(5_000_000_000u64)).unwrap(),
            "te6cckEBAgEAOQABZllfB7wAAAAAAAAAAFASoF8gCAEdDpb1s3fXf4kQIXEnOTHqCwBI3jMwQIG3lkp4mHte8QEAASDleV8G"
        );
    }

    #[test]
    fn test_sign_earn() {
        let signer = test_signer();
        let provider = YieldProvider::Tonstakers.delegation_validator(Chain::Ton);
        let input = signer_input(EarnType::Deposit(provider), build_stake_payload_base64().unwrap(), "10000");

        let signed = signer.sign_earn(&input, Some(1_000_000_000)).unwrap();

        assert_eq!(
            signed,
            vec![
                "te6cckEBBAEAxAABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGcyRclzA0Phkl4EMv6ba3GIp6Tx3f1PU9foE3I/W8msQmi8NMYR7h0v8L2OlXLMvqvJ+WWJMk/D/s+CyNQcpFQASmpoxc7msoAAAAAAQADAgFoYgASwA6bnRklOr1y4MxDEh82TpZnlC7Kl8tiVkz/uVEgGiHc14iAAAAAAAAAAAAAAAAAAQMAKEfVQ5EAAAAAAAAAAUdFTQBp7vEnTx0W9Q==".to_string()
            ]
        );
    }
}
