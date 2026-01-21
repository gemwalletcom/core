use serde::de;

fn map_err<E, T>(value: Result<T, String>) -> Result<T, E>
where
    E: de::Error,
{
    value.map_err(de::Error::custom)
}

mod string_or_number;
mod string_value;

pub(crate) use string_or_number::{StringOrNumberFromValue, StringOrNumberVisitor};
pub(crate) use string_value::StringFromValueVisitor;
