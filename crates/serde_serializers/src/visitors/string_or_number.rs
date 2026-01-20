use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Visitor};

use super::map_err;

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

    fn map_value<E>(value: Result<T, String>) -> Result<T, E>
    where
        E: de::Error,
    {
        map_err(value)
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
        Self::map_value(T::from_str(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_value(T::from_str(&value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_value(T::from_u64(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_value(T::from_i64(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_value(T::from_f64(value))
    }
}
