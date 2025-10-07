use base64::prelude::{BASE64_URL_SAFE_NO_PAD, Engine};
use crc::Crc;

type Workchain = i32;
type HashPart = [u8; 32];

fn crc16(slice: &[u8]) -> u16 {
    Crc::<u16>::new(&crc::CRC_16_XMODEM).checksum(slice)
}

pub struct ParseError(pub String);

fn encode_base64(workchain: Workchain, hash_part: &HashPart) -> String {
    let mut buffer = [0u8; 36];

    buffer[0] = 0x11;
    buffer[1] = (workchain & 0xFF) as u8;
    buffer[2..34].clone_from_slice(hash_part);

    let crc = crc16(&buffer[0..34]);
    buffer[34] = ((crc >> 8) & 0xFF) as u8;
    buffer[35] = (crc & 0xFF) as u8;

    BASE64_URL_SAFE_NO_PAD.encode(buffer)
}

pub struct Address {
    workchain: Workchain,
    hash_part: HashPart,
}

impl Address {
    pub fn new(workchain: Workchain, hash_part: HashPart) -> Self {
        Self { workchain, hash_part }
    }

    pub fn get_hash_part(&self) -> &HashPart {
        &self.hash_part
    }

    pub fn from_hex_str<S>(hex_str: S) -> Result<Self, ParseError>
    where
        S: AsRef<str>,
    {
        let raw = hex_str.as_ref();
        let parts: Vec<&str> = raw.split(':').collect();

        if parts.len() != 2 {
            return Err(ParseError("Invalid address format".to_string()));
        }

        let workchain = parts[0].parse::<i32>().map_err(|_| ParseError("Invalid workchain".to_string()))?;

        let hash_part = hex::decode(parts[1]).map_err(|_| ParseError("Invalid hash".to_string()))?;

        if hash_part.len() != 32 {
            return Err(ParseError("Invalid hash length".to_string()));
        }

        Ok(Self {
            workchain,
            hash_part: hash_part.as_slice().try_into().unwrap(),
        })
    }

    pub fn to_base64_url(&self) -> String {
        encode_base64(self.workchain, &self.hash_part)
    }
}

pub fn hex_to_base64_address(hex_str: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Address::from_hex_str(&hex_str)?.to_base64_url())
}

pub fn base64_to_hex_address(base64_str: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use base64::prelude::{BASE64_STANDARD_NO_PAD, BASE64_URL_SAFE_NO_PAD};

    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(&base64_str)
        .or_else(|_| BASE64_STANDARD_NO_PAD.decode(&base64_str))
        .map_err(|_| ParseError("Invalid base64".to_string()))?;

    if bytes.len() != 36 {
        return Err(ParseError("Invalid base64 length".to_string()).into());
    }

    let workchain = bytes[1] as i32;
    let hash = &bytes[2..34];

    Ok(format!("{}:{}", workchain, hex::encode(hash)))
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_to_base64() {
        let raw = "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78";
        let address = Address::from_hex_str(raw).unwrap();

        assert_eq!(address.to_base64_url(), "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8");
    }

    #[test]
    fn test_parse_address() {
        let hex = "0:0e97797708411c29a3cb1f3f810ef4f83f41d990838f7f93ce7082c4ff9aa026";
        let address = Address::from_hex_str(hex).unwrap();

        assert_eq!(address.to_base64_url(), "EQAOl3l3CEEcKaPLHz-BDvT4P0HZkIOPf5POcILE_5qgJuR2");
    }

    #[test]
    fn test_hex_to_base64_address() {
        let addr = "0:8c50a91220a5ccf086a1b2113b1a78787555f02b20d3fa6e97ba1acd710dbdaa";
        let result = hex_to_base64_address(addr.to_string()).unwrap();

        assert_eq!(result, "EQCMUKkSIKXM8IahshE7Gnh4dVXwKyDT-m6XuhrNcQ29qvOh");
    }

    #[test]
    fn test_invalid_format() {
        let result = Address::from_hex_str("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_workchain() {
        let result = Address::from_hex_str("abc:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_hash() {
        let result = Address::from_hex_str("0:invalid_hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_hash_length() {
        let result = Address::from_hex_str("0:abcd1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_to_hex_address() {
        let base64 = "EQCOh0t62bvrv8SIELiTnJj1BYAkbxmYIEDbyyU8TD2veND8";
        let hex = base64_to_hex_address(base64.to_string()).unwrap();

        assert_eq!(hex, "0:8e874b7ad9bbebbfc48810b8939c98f50580246f19982040dbcb253c4c3daf78");
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_hex = "0:0e97797708411c29a3cb1f3f810ef4f83f41d990838f7f93ce7082c4ff9aa026";
        let base64 = hex_to_base64_address(original_hex.to_string()).unwrap();
        let decoded_hex = base64_to_hex_address(base64).unwrap();

        assert_eq!(original_hex, decoded_hex);
    }
}
