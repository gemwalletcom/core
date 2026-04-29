use std::str::FromStr;

use num_bigint::BigUint;
use primitives::{EarnType, SignerError};

use crate::{
    Address,
    tvm::{BagOfCells, CellBuilder},
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
    Ok(BagOfCells::from_root(builder.build()?).to_base64(true)?)
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
    Ok(BagOfCells::from_root(builder.build()?).to_base64(true)?)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use primitives::{Asset, Chain, ContractCallData, EarnType, SignerInput, TransactionInputType, TransactionLoadMetadata, YieldProvider};

    use super::{build_stake_payload_base64, build_unstake_payload_base64};
    use crate::{
        Address,
        signer::{TonSigner, testkit::TEST_ADDRESS as TON_TEST_WALLET_ADDRESS},
    };

    const TEST_TON_PRIVATE_KEY: &str = "c7702dadcd00d470df27dee0ddd97fbcf9deba52b60f7dd2b296ff42bb1fcad6";

    #[test]
    fn test_build_stake_payload_base64() {
        assert_eq!(build_stake_payload_base64().unwrap(), "te6cckEBAQEAFgAAKEfVQ5EAAAAAAAAAAUdFTQBp7vEnOxjgfw==");
    }

    #[test]
    fn test_build_unstake_payload_base64() {
        let owner = Address::parse(TON_TEST_WALLET_ADDRESS).unwrap();
        assert_eq!(
            build_unstake_payload_base64(&owner, &BigUint::from(5_000_000_000u64)).unwrap(),
            "te6cckEBAgEAOQABZllfB7wAAAAAAAAAAFASoF8gCACxq4qfdwkRXv1VoZuOs5Ue3+8/kqqiDYJnNgb9gUgGjwEAASDYnkB8"
        );
    }

    #[test]
    fn test_sign_earn() {
        let private_key = hex::decode(TEST_TON_PRIVATE_KEY).unwrap();
        let signer = TonSigner::new(&private_key).unwrap();
        let provider = YieldProvider::Tonstakers.delegation_validator(Chain::Ton);
        let input_type = TransactionInputType::Earn(
            Asset::from_chain(Chain::Ton),
            EarnType::Deposit(provider),
            ContractCallData::new(TON_TEST_WALLET_ADDRESS.to_string(), build_stake_payload_base64().unwrap()),
        );
        let input = SignerInput::mock_with_input_type(input_type, "", "", "10000", TransactionLoadMetadata::mock_ton(1));

        let signed = signer.sign_earn(&input, Some(1_000_000_000)).unwrap();

        assert_eq!(
            signed,
            vec![
                "te6cckEBBAEAxAABRYgBkF1w67cBLG0e0D7j0y2ShzflCe2JrlAjS4pC8UHg85AMAQGc586+/tdveRSv7FpL3QNz4uEs7fsaNih6wD1u9UeqSjlwvKknRCpL4kxqMzR67FdLNQ3eeriBeKX7dJEgRPopDimpoxc7msoAAAAAAQADAgFoYgAsauKn3cJEV79VaGbjrOVHt/vP5Kqog2CZzYG/YFIBo6Hc14iAAAAAAAAAAAAAAAAAAQMAKEfVQ5EAAAAAAAAAAUdFTQBp7vEn4cjcXg==".to_string()
            ]
        );
    }
}
