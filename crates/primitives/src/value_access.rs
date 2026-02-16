use serde_json::Value;

pub trait ValueAccess {
    fn get_value(&self, key: &str) -> Result<&Value, String>;
    fn at(&self, index: usize) -> Result<&Value, String>;
    fn string(&self) -> Result<&str, String>;
}

impl ValueAccess for Value {
    fn get_value(&self, key: &str) -> Result<&Value, String> {
        self.get(key).ok_or_else(|| format!("Missing {} parameter", key))
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
