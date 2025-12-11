#[uniffi::export]
pub fn create_referral_auth_message(address: &str, chain_id: u64) -> String {
    referral::create_siwe_message(address, chain_id)
}
