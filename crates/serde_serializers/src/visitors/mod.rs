use serde::de;

fn map_err<E, T>(value: Result<T, String>) -> Result<T, E>
where
    E: de::Error,
{
    value.map_err(de::Error::custom)
}

fn map_err_option<E, T>(value: Result<T, String>) -> Result<Option<T>, E>
where
    E: de::Error,
{
    map_err(value).map(Some)
}

mod number_value;
mod string_or_number;
mod string_value;

pub(crate) use number_value::{NumberFromValue, OptionNumberVisitor};
pub(crate) use string_or_number::{StringOrNumberFromValue, StringOrNumberVisitor};
pub(crate) use string_value::StringFromValueVisitor;
