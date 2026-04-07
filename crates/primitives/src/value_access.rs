use serde::de::DeserializeOwned;
use serde_json::Value;

pub trait JsonDecode {
    fn decode<T: DeserializeOwned>(&self) -> Option<T>;
}

impl JsonDecode for Option<Value> {
    fn decode<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.clone()?).ok()
    }
}

pub trait ValueAccess {
    fn get_value(&self, key: &str) -> Result<&Value, String>;
    fn get_string(&self, key: &str) -> Result<&str, String>;
    fn get_i64(&self, key: &str) -> Result<i64, String>;
    fn at(&self, index: usize) -> Result<&Value, String>;
    fn string(&self) -> Result<&str, String>;
}

impl ValueAccess for Value {
    fn get_value(&self, key: &str) -> Result<&Value, String> {
        self.get(key).ok_or_else(|| format!("Missing {} parameter", key))
    }

    fn get_string(&self, key: &str) -> Result<&str, String> {
        self.get_value(key)?.string()
    }

    fn get_i64(&self, key: &str) -> Result<i64, String> {
        self.get_value(key)?.as_i64().ok_or_else(|| format!("Expected integer value for {}", key))
    }

    fn at(&self, index: usize) -> Result<&Value, String> {
        self.as_array()
            .and_then(|array| array.get(index))
            .ok_or_else(|| format!("Missing parameter at index {}", index))
    }

    fn string(&self) -> Result<&str, String> {
        self.as_str().ok_or_else(|| "Expected string value".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ValueAccess;

    #[test]
    fn test_keyed_accessors() {
        let value = serde_json::json!({
            "name": "USDT",
            "points": 650,
        });

        assert_eq!(value.get_string("name").unwrap(), "USDT");
        assert_eq!(value.get_i64("points").unwrap(), 650);
    }
}
