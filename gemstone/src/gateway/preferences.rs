use super::GatewayError;
use std::sync::Arc;

#[uniffi::export(with_foreign)]
pub trait GemPreferences: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, GatewayError>;
    fn set(&self, key: String, value: String) -> Result<(), GatewayError>;
    fn remove(&self, key: String) -> Result<(), GatewayError>;
}

pub(crate) struct PreferencesWrapper {
    pub(crate) preferences: Arc<dyn GemPreferences>,
}

impl primitives::Preferences for PreferencesWrapper {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.get(key).map_err(Into::into)
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.set(key, value).map_err(Into::into)
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.remove(key).map_err(Into::into)
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub struct EmptyPreferences;

#[cfg(test)]
impl GemPreferences for EmptyPreferences {
    fn get(&self, _key: String) -> Result<Option<String>, GatewayError> {
        Ok(None)
    }

    fn set(&self, _key: String, _value: String) -> Result<(), GatewayError> {
        Ok(())
    }

    fn remove(&self, _key: String) -> Result<(), GatewayError> {
        Ok(())
    }
}
