use bech32::{convert_bits, FromBase32, ToBase32};

fn convert_cosmos_address(address: &str, hrp: &str) -> Result<String, Error> {
    let (hrp_cosmos, data_cosmos) = bech32::decode(address)?;
    let data_bytes =
        bech32::FromBase32::from_base32(&data_cosmos)?;

    let mut data_bits = convert_bits(&data_bytes, 5, 8, false)?;
    data_bits.insert(0, 0);

    let data_chain_bytes =
        convert_bits(&data_bits, 8, 5, false)?;
    let data_chain = bech32::ToBase32::to_base32(&data_chain_bytes);

    bech32::encode(hrp, data_chain)?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_address_convert_expected_case() {
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj";
        let osmosis_hrp = "osmosis";

        let expected = "osmosis1h3laqcrmul79zwtw6j63ncsl0adfj07wm8vf00";

        let output_address = convert_cosmos_address(cosmos_address, osmosis_hrp);
        assert_eq(expected, output_address);
    }

    #[test]
    fn test_cosmos_address_convert_unexpected_case() {
        let cosmos_address = "cosmos1h3laqcrmul79zwtw6j63ncsl0adfj07wgupylj";
        let osmosis_hrp = "gemstone";

        let expected = "osmosis1h3laqcrmul79zwtw6j63ncsl0adfj07wm8vf00";

        let output_address = convert_cosmos_address(cosmos_address, osmosis_hrp);
        assert_nq(expected, output_address);
    }
}
