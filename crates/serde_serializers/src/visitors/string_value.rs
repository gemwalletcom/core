use std::fmt;

use serde::de::{self, Visitor};

pub(crate) struct StringFromValueVisitor {
    allow_null: bool,
}

impl StringFromValueVisitor {
    pub(crate) fn new(allow_null: bool) -> Self {
        Self { allow_null }
    }

    fn expected_message(&self) -> &'static str {
        if self.allow_null { "a string, number, or null" } else { "a string or number" }
    }

    fn null_value<E>(self) -> Result<String, E>
    where
        E: de::Error,
    {
        if self.allow_null {
            Ok(String::new())
        } else {
            Err(de::Error::custom(format!("expected {}", self.expected_message())))
        }
    }
}

impl<'de> Visitor<'de> for StringFromValueVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.expected_message())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_owned())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_string())
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_string())
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_string())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.null_value()
    }
}
