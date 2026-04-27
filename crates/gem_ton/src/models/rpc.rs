use std::error::Error;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::biguint_from_hex_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunGetMethodResult {
    pub stack: Vec<RunGetMethodStackItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RunGetMethodStackItem {
    Num {
        value: String,
    },
    #[serde(other)]
    Other,
}

impl RunGetMethodResult {
    pub fn get_num(&self, index: usize) -> Result<BigUint, Box<dyn Error + Send + Sync>> {
        match self.stack.get(index) {
            Some(RunGetMethodStackItem::Num { value }) => biguint_from_hex_str(value),
            _ => Err(format!("expected num at TON stack index {index}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_get_num() {
        let value = json!({ "stack": [{ "type": "num", "value": "0xff" }, { "type": "cell", "value": "..." }] });
        let result: RunGetMethodResult = serde_json::from_value(value).unwrap();

        assert_eq!(result.get_num(0).unwrap(), BigUint::from(255u32));
        assert!(result.get_num(1).is_err());
        assert!(result.get_num(2).is_err());
    }
}
