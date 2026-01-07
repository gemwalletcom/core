#[derive(Debug, Clone)]
pub struct IpCheckResult {
    pub ip_address: String,
    pub country_code: String,
    pub confidence_score: i64,
    pub is_tor: bool,
    pub usage_type: String,
    pub isp: String,
}
