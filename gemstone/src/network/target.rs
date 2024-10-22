use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, uniffi::Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}
