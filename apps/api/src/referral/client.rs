use primitives::{Referral, ReferralCodeRequest, ReferralEventItem};
use storage::Database;

pub struct ReferralClient {
    database: Database,
}

impl ReferralClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_referral(&mut self, address: &str) -> Result<Referral, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_by_address(address)?)
    }

    pub fn get_referral_events(&mut self, address: &str) -> Result<Vec<ReferralEventItem>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub fn create_referral(&mut self, request: &ReferralCodeRequest) -> Result<Referral, Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        Ok(self.database.client()?.rewards().create_reward(&address, &request.code)?)
    }

    pub fn use_referral_code(&mut self, request: &ReferralCodeRequest) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        self.database.client()?.rewards().use_referral_code(&address, &request.code)?;
        Ok(())
    }

    fn verify_request(&self, request: &ReferralCodeRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !referral::verify_siwe_signature(&request.message, &request.signature, &request.address) {
            return Err("Invalid signature".into());
        }

        let message = referral::parse_siwe_message(&request.message).ok_or("Invalid message format")?;
        Ok(message.address)
    }
}
