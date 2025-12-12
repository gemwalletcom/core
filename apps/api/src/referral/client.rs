use primitives::{Rewards, RewardsEventItem, RewardsReferralRequest};
use storage::Database;

pub struct RewardsClient {
    database: Database,
}

impl RewardsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_rewards(&mut self, address: &str) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_by_address(address)?)
    }

    pub fn get_rewards_events(&mut self, address: &str) -> Result<Vec<RewardsEventItem>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.database.client()?.rewards().get_reward_events_by_address(address)?)
    }

    pub fn create_referral(&mut self, request: &RewardsReferralRequest) -> Result<Rewards, Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        Ok(self.database.client()?.rewards().create_reward(&address, &request.code)?)
    }

    pub fn use_referral_code(&mut self, request: &RewardsReferralRequest) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let address = self.verify_request(request)?;
        self.database.client()?.rewards().use_referral_code(&address, &request.code)?;
        Ok(())
    }

    fn verify_request(&self, request: &RewardsReferralRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !referral::verify_siwe_signature(&request.message, &request.signature, &request.address) {
            return Err("Invalid signature".into());
        }

        let message = referral::parse_siwe_message(&request.message).ok_or("Invalid message format")?;
        Ok(message.address)
    }
}
