/// This function simply decodes the bech32 address and encodes it with a different hrp.
/// The caller is responsible for validating the data and hrp, more specifically, in the case of
/// Cosmos, the encoded data is sha256 hash of the public key.
pub fn convert_cosmos_address(address: &str, hrp: &str) -> Result<String, anyhow::Error> {
    let (_, decoded) = bech32::decode(address)?;
    let new_hrp = bech32::hrp::Hrp::parse(hrp)?;
    let encoded = bech32::encode::<bech32::Bech32>(new_hrp, decoded.as_slice())?;

    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_address_convert() {
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj";
        let expected = "osmosis1h3laqcrmul79zwtw6j63ncsl0adfj07wm8vf00";

        let output_address = convert_cosmos_address(cosmos_address, "osmosis").unwrap();
        assert_eq!(expected, output_address);
    }

    #[test]
    fn test_invalid_cosmos_address() {
        // invalid checksum
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgu";

        let result = convert_cosmos_address(cosmos_address, "osmosis");
        assert!(result.is_err());
    }
}
