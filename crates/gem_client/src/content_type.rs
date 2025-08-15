use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
}

impl ContentType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => "application/json",
            ContentType::TextPlain => "text/plain",
        }
    }
}

impl FromStr for ContentType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/json" => Ok(ContentType::ApplicationJson),
            "text/plain" => Ok(ContentType::TextPlain),
            _ => Err("Unknown content type"),
        }
    }
}