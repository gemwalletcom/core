use primitives::IpUsageType;
use serde::Deserialize;

use crate::model::IpCheckResult;

#[derive(Debug, Clone, Deserialize)]
pub struct IpApiResponse {
    pub ip: String,
    pub is_tor: Option<bool>,
    pub is_proxy: Option<bool>,
    pub is_vpn: Option<bool>,
    pub is_abuser: Option<bool>,
    pub company: Option<IpApiCompany>,
    pub asn: Option<IpApiAsn>,
    pub location: Option<IpApiLocation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpApiCompany {
    pub name: Option<String>,
    pub abuser_score: Option<String>,
    #[serde(rename = "type")]
    pub company_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpApiAsn {
    pub abuser_score: Option<String>,
    pub org: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IpApiLocation {
    pub country_code: Option<String>,
}

impl IpApiResponse {
    pub fn as_ip_check_result(&self) -> IpCheckResult {
        IpCheckResult {
            ip_address: self.ip.clone(),
            country_code: self.location.as_ref().and_then(|l| l.country_code.clone()).unwrap_or_default(),
            confidence_score: self.calculate_confidence_score(),
            is_tor: self.is_tor.unwrap_or(false),
            is_vpn: self.is_vpn.unwrap_or(false),
            usage_type: self.determine_usage_type(),
            isp: self
                .company
                .as_ref()
                .and_then(|c| c.name.clone())
                .or_else(|| self.asn.as_ref().and_then(|a| a.org.clone()))
                .unwrap_or_default(),
        }
    }

    fn determine_usage_type(&self) -> IpUsageType {
        self.company
            .as_ref()
            .and_then(|c| c.company_type.as_deref())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    fn calculate_confidence_score(&self) -> i64 {
        let abuser_score = self.parse_abuser_score();
        let mut score = (abuser_score * 100.0).round() as i64;

        if self.is_abuser.unwrap_or(false) {
            score = score.max(50);
        }

        if self.is_proxy.unwrap_or(false) || self.is_vpn.unwrap_or(false) {
            score = score.max(25);
        }

        if self.is_tor.unwrap_or(false) {
            score = score.max(75);
        }

        score.clamp(0, 100)
    }

    fn parse_abuser_score(&self) -> f64 {
        let score_str = self
            .company
            .as_ref()
            .and_then(|c| c.abuser_score.clone())
            .or_else(|| self.asn.as_ref().and_then(|a| a.abuser_score.clone()));

        if let Some(s) = score_str {
            if let Some(num_str) = s.split_whitespace().next() {
                if let Ok(val) = num_str.parse::<f64>() {
                    return val;
                }
            }
        }

        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_abuser_score() {
        let response = IpApiResponse {
            ip: "1.2.3.4".to_string(),
            is_tor: None,
            is_proxy: None,
            is_vpn: None,
            is_abuser: None,
            company: Some(IpApiCompany {
                name: None,
                abuser_score: Some("0.0044 (Low)".to_string()),
                company_type: None,
            }),
            asn: None,
            location: None,
        };

        let score = response.parse_abuser_score();
        assert!((score - 0.0044).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_confidence_score_low() {
        let response = IpApiResponse {
            ip: "1.2.3.4".to_string(),
            is_tor: None,
            is_proxy: None,
            is_vpn: None,
            is_abuser: None,
            company: Some(IpApiCompany {
                name: None,
                abuser_score: Some("0.0044 (Low)".to_string()),
                company_type: None,
            }),
            asn: None,
            location: None,
        };

        assert_eq!(response.calculate_confidence_score(), 0);
    }

    #[test]
    fn test_calculate_confidence_score_tor() {
        let response = IpApiResponse {
            ip: "1.2.3.4".to_string(),
            is_tor: Some(true),
            is_proxy: None,
            is_vpn: None,
            is_abuser: None,
            company: None,
            asn: None,
            location: None,
        };

        assert_eq!(response.calculate_confidence_score(), 75);
    }

    #[test]
    fn test_determine_usage_type_hosting() {
        let response = IpApiResponse {
            ip: "1.2.3.4".to_string(),
            is_tor: None,
            is_proxy: None,
            is_vpn: None,
            is_abuser: None,
            company: Some(IpApiCompany {
                name: None,
                abuser_score: None,
                company_type: Some("hosting".to_string()),
            }),
            asn: None,
            location: None,
        };

        assert_eq!(response.determine_usage_type(), IpUsageType::Hosting);
    }
}
