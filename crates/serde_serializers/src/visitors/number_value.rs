use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Visitor};

use super::map_err_option;

pub(crate) trait NumberFromValue: Sized {
    fn from_u64(value: u64) -> Result<Self, String>;
    fn from_i64(value: i64) -> Result<Self, String>;
}

pub(crate) struct OptionNumberVisitor<T>(PhantomData<T>);

impl<T> OptionNumberVisitor<T> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    fn map_number<E>(value: Result<T, String>) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        map_err_option(value)
    }
}

impl<'de, T> Visitor<'de> for OptionNumberVisitor<T>
where
    T: NumberFromValue,
{
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or null")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_number(T::from_u64(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::map_number(T::from_i64(value))
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
