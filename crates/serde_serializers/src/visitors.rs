use std::fmt;
use std::marker::PhantomData;

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

pub(crate) trait NumberFromValue: Sized {
    fn from_u64(value: u64) -> Result<Self, String>;
    fn from_i64(value: i64) -> Result<Self, String>;
    fn from_f64(value: f64) -> Result<Self, String>;
}

pub(crate) struct OptionNumberVisitor<T>(PhantomData<T>);

impl<T> OptionNumberVisitor<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> Visitor<'de> for OptionNumberVisitor<T>
where
    T: NumberFromValue,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a number or null")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_u64(value).map(Some).map_err(de::Error::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_i64(value).map(Some).map_err(de::Error::custom)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_f64(value).map(Some).map_err(de::Error::custom)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }
}

pub(crate) trait StringOrNumberFromValue: Sized {
    const EXPECTING: &'static str;

    fn from_str(value: &str) -> Result<Self, String>;
    fn from_u64(value: u64) -> Result<Self, String>;
    fn from_i64(value: i64) -> Result<Self, String>;
    fn from_f64(value: f64) -> Result<Self, String>;
}

pub(crate) struct StringOrNumberVisitor<T>(PhantomData<T>);

impl<T> StringOrNumberVisitor<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> Visitor<'de> for StringOrNumberVisitor<T>
where
    T: StringOrNumberFromValue,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(T::EXPECTING)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_str(value).map_err(de::Error::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_str(&value).map_err(de::Error::custom)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_u64(value).map_err(de::Error::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_i64(value).map_err(de::Error::custom)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_f64(value).map_err(de::Error::custom)
    }
}
