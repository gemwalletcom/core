// Taken from https://github.com/metaplex-foundation/mpl-token-metadata/blob/main/programs/token-metadata/program/src/state/metadata.rs
mod collection;
mod data;
mod uses;

pub mod metadata;
use crate::metaplex::metadata::Metadata;
use base64::{engine::general_purpose, Engine as _};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
    TokenRecord,
    MetadataDelegate,
    EditionMarkerV2,
    HolderDelegate,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenStandard {
    NonFungible,                    // This is a master edition
    FungibleAsset,                  // A token with metadata that can also have attributes
    Fungible,                       // A token with simple metadata
    NonFungibleEdition,             // This is a limited edition
    ProgrammableNonFungible,        // NonFungible with programmable configuration
    ProgrammableNonFungibleEdition, // NonFungible with programmable configuration
}

pub fn decode_metadata(base64_str: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let data = general_purpose::STANDARD.decode(base64_str)?;
    let metadata = Metadata::deserialize(&mut data.as_slice())?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use crate::{
        metaplex::{decode_metadata, metadata::Metadata, Key},
        pubkey::Pubkey,
    };
    use std::str::FromStr;

    #[test]
    fn test_metadata_data() {
        let string = "BBzjWe1aAS4E+hQrnHUaHF6Hz9CgFhuchf/TG3jN/Nj2xvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWEgAAAAVVNEIENvaW4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKAAAAVVNEQwAAAAAAAMgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAfwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        let metadata = decode_metadata(string).unwrap();

        assert_eq!(metadata.key, Key::MetadataV1);
        assert_eq!(metadata.mint.to_string(), "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

        assert_eq!(metadata.data.uri.trim_matches(char::from(0)), "");
        assert_eq!(metadata.data.symbol.trim_matches(char::from(0)), "USDC");
        assert_eq!(metadata.data.name.trim_matches(char::from(0)), "USD Coin");
    }

    #[test]
    fn test_find_pda() {
        let mut mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let (pda, _) = Metadata::find_pda(mint).unwrap();

        assert_eq!(pda.to_string(), "5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq");

        mint = Pubkey::from_str("MEW1gQWJ3nEXg2qgERiKu7FAFj79PHvQVREQUzScPP5").unwrap();
        let (pda, _) = Metadata::find_pda(mint).unwrap();

        assert_eq!(pda.to_string(), "5G95zJ9w6ESv7AFWqLKNfbZAoKBADjpVm9MT1cQm8Dpw");
    }
}
