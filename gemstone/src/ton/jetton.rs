use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use gem_ton::address::TonAddress;
use gem_ton::cell::{BagOfCells, Cell, CellBuilder};

pub fn encode_wallet_address_data(address: &str) -> Result<String, anyhow::Error> {
    let mut writer = CellBuilder::new();
    let addr = TonAddress::from_base64_url(address)?;
    let cell = writer.store_address(&addr)?.build()?;
    let boc = BagOfCells::from_root(cell);
    let encoded = boc.serialize(true)?;
    Ok(STANDARD.encode(encoded))
}

pub fn decode_address_data(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cell = Cell {
        data: STANDARD.decode(data)?,
        bit_len: 267,
        references: vec![],
    };
    let mut reader = cell.parser();
    let addr = reader.load_address()?;
    Ok(addr.to_base64_url())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_wallet_address_data() {
        let address = "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV";
        let result = encode_wallet_address_data(address).unwrap();

        assert_eq!(
            result,
            "te6cckEBAQEAJAAAQ4AGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+fAmffsx"
        );

        let address = "EQBvI0aFLnw2QbZgjMPCLRdtRHxhUyinQudg6sdiohIwg5jL";
        let result = encode_wallet_address_data(address).unwrap();

        assert_eq!(
            result,
            "te6cckEBAQEAJAAAQ4AN5GjQpc+GyDbMEZh4RaLtqI+MKmUU6FzsHVjsVEJGEHDW8Lb+"
        );
    }

    #[test]
    fn test_decode_address_data() {
        let data = "gA057dpDOFWrunYZ0EZRwnbhuaQwX9taKLFu/2/cN8gDQA==";
        let result = decode_address_data(data).unwrap();

        assert_eq!(result, "EQBpz27SGcKtXdOwzoIyjhO3Dc0hgv7a0UWLd_t-4b5AGrg6");

        let data = "gBpa/IXTav3vLvznbIUL0fFS7uTxUc4ZWs74s3fPGz7uIA==";
        let result = decode_address_data(data).unwrap();

        assert_eq!(result, "EQDS1-Qum1fveXfnO2QoXo-Kl3cnio5wytZ3xZu-eNn3cbsY");
    }
}
