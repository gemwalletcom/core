use bigdecimal::BigDecimal;
use num_bigint::BigInt;

pub struct EtherConv {}

impl EtherConv {
    pub fn one() -> BigInt {
        BigInt::from(10u64.pow(18))
    }

    /// Parse Ether to Wei as BigInt
    pub fn parse_ether(ether: &str) -> BigInt {
        to_bn_wei(ether, 18)
    }

    /// Format Wei to Ether as String
    pub fn format_ether(wei: &BigInt) -> String {
        from_bn_wei(wei, 18)
    }
}

pub fn to_bn_wei(value: &str, decimals: u32) -> BigInt {
    let ether_value = value.parse::<BigDecimal>().unwrap();
    let wei_value = (&ether_value * BigDecimal::from(10u64.pow(decimals))).with_scale(0);

    wei_value.as_bigint_and_exponent().0
}

pub fn from_bn_wei(wei: &BigInt, decimals: u32) -> String {
    let wei_decimal = BigDecimal::from(wei.clone());
    let divisor = BigDecimal::from(10u64.pow(decimals));
    let ether_value = wei_decimal.with_scale(decimals as i64) / divisor;

    let mut result = ether_value.to_string();
    if result.contains('.') {
        result = result.trim_end_matches('0').trim_end_matches('.').to_string();
    }

    // bigdecimal returns scientific notation for very small numbers (e.g. 1e-18)
    if result.to_lowercase().contains("e-") {
        let parts: Vec<&str> = result.split(['e', 'E']).collect();
        let base = parts[0].trim_end_matches('0').trim_end_matches('.');
        let exponent = parts[1][1..].parse::<usize>().unwrap_or(0);

        result = format!("0.{}{}", "0".repeat(exponent - 1), base);
        result = result.trim_end_matches('0').to_string();
    }

    result
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ether_conversion() {
        let ether = "0.0001";
        let wei = EtherConv::parse_ether(ether);

        assert_eq!(wei.to_string(), "100000000000000");

        let ether = "1500.123";
        let wei = EtherConv::parse_ether(ether);

        assert_eq!(wei.to_string(), "1500123000000000000000");
    }

    #[test]
    fn test_wei_to_ether_conversion() {
        let wei = BigInt::parse_bytes(b"100000000000000", 10).unwrap();
        let ether = EtherConv::format_ether(&wei);

        assert_eq!(ether, "0.0001");

        let wei = BigInt::parse_bytes(b"1500123000000000000000", 10).unwrap();
        let ether = EtherConv::format_ether(&wei);

        assert_eq!(ether, "1500.123");

        // Test with whole number
        let wei = BigInt::parse_bytes(b"1000000000000000000", 10).unwrap();
        let ether = EtherConv::format_ether(&wei);

        assert_eq!(ether, "1");

        // Test with very small number
        let wei = BigInt::parse_bytes(b"1", 10).unwrap();
        let ether = EtherConv::format_ether(&wei);

        assert_eq!(ether, "0.000000000000000001");
    }
}
