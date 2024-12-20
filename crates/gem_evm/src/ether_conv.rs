use bigdecimal::BigDecimal;
use num_bigint::BigInt;

pub struct EtherConv {}

impl EtherConv {
    pub fn one() -> BigInt {
        BigInt::from(10u64.pow(18))
    }

    /// Parse Ether to Wei as BigInt
    pub fn parse_ether(ether: &str) -> BigInt {
        let ether_value = ether.parse::<BigDecimal>().unwrap();
        let wei_value = (&ether_value * BigDecimal::from(10u64.pow(18))).with_scale(0);

        wei_value.as_bigint_and_exponent().0
    }
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
}
