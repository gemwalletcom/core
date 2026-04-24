use super::{BagOfCells, CellBuilder, TonSigner};

pub const TEST_ADDRESS: &str = "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg";
pub const TEST_PRIVATE_KEY: &str = "1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34";
pub const TEST_PUBLIC_KEY: &str = "d369452197c2a56481e5e2d3e8bf03de2349f67a63151956822208c2334adee2";

pub fn mock_signer() -> TonSigner {
    let private_key = hex::decode(TEST_PRIVATE_KEY).unwrap();
    TonSigner::new(&private_key).unwrap()
}

pub fn mock_signer_address() -> String {
    use primitives::Address;
    mock_signer().address().encode()
}

pub fn mock_cell() -> String {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, 0).unwrap();
    BagOfCells::from_root(builder.build().unwrap()).to_base64(true).unwrap()
}
