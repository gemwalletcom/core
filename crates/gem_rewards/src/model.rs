use primitives::IpUsageType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCheckResult {
    pub ip_address: String,
    pub country_code: String,
    pub confidence_score: i64,
    pub is_tor: bool,
    pub is_vpn: bool,
    pub usage_type: IpUsageType,
    pub isp: String,
}
