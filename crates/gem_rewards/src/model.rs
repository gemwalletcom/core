#[derive(Debug, Clone)]
pub struct IpCheckConfig {
    pub confidence_score_threshold: i64,
    pub ineligible_usage_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IpCheckResult {
    pub ip_address: String,
    pub country_code: String,
    pub confidence_score: i64,
    pub is_tor: bool,
    pub is_blocked: bool,
    pub usage_type: String,
    pub isp: String,
}

impl IpCheckResult {
    pub fn is_suspicious(&self, config: &IpCheckConfig) -> bool {
        if self.confidence_score >= config.confidence_score_threshold {
            return true;
        }
        if self.is_tor || self.is_blocked {
            return true;
        }
        false
    }
}
