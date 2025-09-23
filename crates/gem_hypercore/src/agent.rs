use alloy_primitives::{Address, hex};
use gem_hash::keccak::keccak256;
use k256::{
    SecretKey,
    elliptic_curve::{rand_core::OsRng, sec1::ToEncodedPoint},
};
use primitives::Preferences;
use std::{error::Error, sync::Arc};

pub struct Agent {
    preferences: Arc<dyn Preferences>,
}

impl Agent {
    pub fn new(preferences: Arc<dyn Preferences>) -> Self {
        Self { preferences }
    }

    fn address_key(&self, sender_address: &str) -> String {
        format!("{}_agent_address", sender_address)
    }

    fn private_key_key(&self, sender_address: &str) -> String {
        format!("{}_agent_key", sender_address)
    }

    pub fn get_or_create_credentials(&self, sender_address: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
        let address_key = self.address_key(sender_address);
        let private_key_key = self.private_key_key(sender_address);

        if let (Some(address), Some(private_key)) = (self.preferences.get(address_key.clone())?, self.preferences.get(private_key_key.clone())?) {
            return Ok((address, private_key));
        }

        self.create_new_credentials(sender_address)
    }

    pub fn regenerate_credentials(&self, sender_address: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
        let address_key = self.address_key(sender_address);
        let private_key_key = self.private_key_key(sender_address);

        self.preferences.remove(address_key)?;
        self.preferences.remove(private_key_key)?;

        self.create_new_credentials(sender_address)
    }

    fn create_new_credentials(&self, sender_address: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
        let agent_private_key = self.generate_private_key()?;
        let agent_address = self.derive_address(&agent_private_key)?;

        let address_key = self.address_key(sender_address);
        let private_key_key = self.private_key_key(sender_address);

        self.preferences.set(address_key, agent_address.clone())?;
        self.preferences.set(private_key_key, agent_private_key.clone())?;

        Ok((agent_address, agent_private_key))
    }

    fn generate_private_key(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng);
        Ok(hex::encode(secret_key.to_bytes()))
    }

    fn derive_address(&self, private_key_hex: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let private_key_bytes = hex::decode(private_key_hex)?;
        let secret_key = SecretKey::from_slice(&private_key_bytes).map_err(|_| "Invalid private key")?;
        let public_key = secret_key.public_key();
        let encoded_point = public_key.to_encoded_point(false);
        let hash = keccak256(&encoded_point.as_bytes()[1..]);
        Ok(Address::from_slice(&hash[12..]).to_string().to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockPreferences {
        data: Mutex<HashMap<String, String>>,
    }

    impl MockPreferences {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl Preferences for MockPreferences {
        fn get(&self, key: String) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
            Ok(self.data.lock().unwrap().get(&key).cloned())
        }

        fn set(&self, key: String, value: String) -> Result<(), Box<dyn Error + Send + Sync>> {
            self.data.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn remove(&self, key: String) -> Result<(), Box<dyn Error + Send + Sync>> {
            self.data.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    #[test]
    fn test_derive_address_known_key() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);
        let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let address = agent.derive_address(private_key).unwrap();

        assert_eq!(address, "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
    }

    #[test]
    fn test_derive_address_another_known_key() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);
        let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
        let address = agent.derive_address(private_key).unwrap();

        assert_eq!(address, "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf");
    }

    #[test]
    fn test_generate_private_key() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);
        let private_key = agent.generate_private_key().unwrap();

        assert_eq!(private_key.len(), 64);
        assert!(private_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_address_derivation_consistency() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);
        let private_key = agent.generate_private_key().unwrap();

        let address1 = agent.derive_address(&private_key).unwrap();
        let address2 = agent.derive_address(&private_key).unwrap();
        assert_eq!(address1, address2);
    }

    #[test]
    fn test_get_or_create_credentials() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);

        let (addr1, key1) = agent.get_or_create_credentials("test_wallet").unwrap();
        let (addr2, key2) = agent.get_or_create_credentials("test_wallet").unwrap();

        assert_eq!(addr1, addr2);
        assert_eq!(key1, key2);
        assert_eq!(addr1.len(), 42);
        assert_eq!(key1.len(), 64);
    }

    #[test]
    fn test_regenerate_credentials() {
        let preferences = Arc::new(MockPreferences::new());
        let agent = Agent::new(preferences);

        let (addr1, key1) = agent.get_or_create_credentials("test_wallet").unwrap();
        let (addr2, key2) = agent.regenerate_credentials("test_wallet").unwrap();

        assert_ne!(addr1, addr2);
        assert_ne!(key1, key2);
    }
}
