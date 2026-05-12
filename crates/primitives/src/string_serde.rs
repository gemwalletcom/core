/// Implements `Serialize` and `Deserialize` for a type that round-trips through its `Display`
/// and `FromStr` impls. Use for typed string ids (e.g. `PriceId`, `WalletId`, `NFTAssetId`).
#[macro_export]
macro_rules! impl_string_serde {
    ($t:ty) => {
        impl serde::Serialize for $t {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $t {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}
