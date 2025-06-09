use anyhow::anyhow;
use primitives::AssetId;

uniffi::custom_type!(AssetId, String, {
    remote,
    lower: |s| s.to_string(),
    try_lift: |s| AssetId::new(&s).ok_or(anyhow!("Invalid AssetId")),
});
