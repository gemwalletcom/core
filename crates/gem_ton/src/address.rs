use crate::TonAddress;

pub fn hex_to_base64_address(hex_str: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let addr = TonAddress::from_hex_str(&hex_str)?;
    Ok(addr.to_base64_url())
}

pub fn base64_to_hex_address(base64_str: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let addr = TonAddress::from_base64_url(&base64_str)?;
    Ok(addr.to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_base64_address() {
        let addr = "0:8c50a91220a5ccf086a1b2113b1a78787555f02b20d3fa6e97ba1acd710dbdaa";
        let result = hex_to_base64_address(addr.to_string()).unwrap();

        assert_eq!(result, "EQCMUKkSIKXM8IahshE7Gnh4dVXwKyDT-m6XuhrNcQ29qvOh");
    }

    #[test]
    fn test_base64_to_hex_address() {
        let non_bounce = "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV";
        let hex_addr = base64_to_hex_address(non_bounce.into()).unwrap();

        assert_eq!(hex_addr, "0:33a14a5a9406979d59b9328898591660b8b1736342b11632efdcc911ab9057cf");

        let bounce = hex_to_base64_address(hex_addr).unwrap();

        assert_eq!(bounce, "EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ");
    }
}
