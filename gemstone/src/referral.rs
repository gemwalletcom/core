#[derive(Debug, Clone, uniffi::Record)]
pub struct ReferralAuthMessage {
    pub message: String,
    pub hash: Vec<u8>,
}

#[uniffi::export]
pub fn create_referral_auth_message(address: &str, chain_id: u64) -> ReferralAuthMessage {
    let message = referral::create_siwe_message(address, chain_id);
    let hash = gem_evm::siwe::eip191_hash(&message).to_vec();
    ReferralAuthMessage { message, hash }
}
