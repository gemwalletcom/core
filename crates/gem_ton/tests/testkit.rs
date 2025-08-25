use gem_ton::rpc::client::TonClient;

pub const TEST_ADDRESS: &str = "EQD-cvR0Nz6XAyRBpDeRqKHEQIAMp2lHFKBWw9bZ-iiERTXn";

pub fn create_ton_test_client() -> TonClient<()> {
    TonClient::new(())
}
