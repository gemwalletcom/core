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
    use primitives::Chain;

    #[test]
    fn test_cosmos_osmosis_convert() {
        let address = "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7";
        let expected = "osmo1fxygpgus4nd5jmfl5j7fh5y8hyy53z8ud07jsv";

        let output_address = convert_cosmos_address(address, Chain::Osmosis.hrp()).unwrap();
        assert_eq!(expected, output_address);
    }

    #[test]
    fn test_injective_dymension_convert() {
        let address = "inj1kgq0kzzatjh0lzv73n0nyvyen6npladdz62dtr";
        let expected = "dym1kgq0kzzatjh0lzv73n0nyvyen6npladd6w30u4";

        let output_address = convert_cosmos_address(address, Chain::Dymension.hrp()).unwrap();
        assert_eq!(expected, output_address);
    }

    #[test]
    fn test_cosmos_saga_convert() {
        let address = "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7";
        let expected = "saga1fxygpgus4nd5jmfl5j7fh5y8hyy53z8um85spc";

        let output_address = convert_cosmos_address(address, Chain::Saga.hrp()).unwrap();
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
