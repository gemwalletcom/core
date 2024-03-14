use bech32::{convert_bits, FromBase32, ToBase32};

fn convert_cosmos_address(address: &str, hrp: &str) -> String {
    let (hrp_cosmos, data_cosmos) = bech32::decode(address).expect("Invalid cosmos address");
    let data_bytes = bech32::FromBase32::from_base32(&data_cosmos).expect("Failed to convert data to bytes");

    let mut data_bits = convert_bits(&data_bytes, 5, 8, false).expect("Failed to convert bits");
    data_bits.insert(0, 0);

    let data_chain_bytes = convert_bits(&data_bits, 8, 5, false).expect("Failed to convert bits back to bytes");
    let data_chain = bech32::ToBase32::to_base32(&data_chain_bytes);

    bech32::encode(hrp, data_chain).expect("Failed to encode chain address")
}
