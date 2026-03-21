pub fn encode_varint(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut v = value;
    while v >= 0x80 {
        buf.push((v as u8) | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
    buf
}

fn field_tag(field_number: u32, wire_type: u8) -> Vec<u8> {
    encode_varint(((field_number as u64) << 3) | wire_type as u64)
}

pub fn encode_varint_field(field_number: u32, value: u64) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }
    [field_tag(field_number, 0), encode_varint(value)].concat()
}

pub fn encode_bytes_field(field_number: u32, data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    [field_tag(field_number, 2), encode_varint(data.len() as u64), data.to_vec()].concat()
}

pub fn encode_string_field(field_number: u32, s: &str) -> Vec<u8> {
    encode_bytes_field(field_number, s.as_bytes())
}

pub fn encode_message_field(field_number: u32, msg: &[u8]) -> Vec<u8> {
    if msg.is_empty() {
        return Vec::new();
    }
    encode_bytes_field(field_number, msg)
}

pub fn encode_coin(denom: &str, amount: &str) -> Vec<u8> {
    [encode_string_field(1, denom), encode_string_field(2, amount)].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_varint() {
        assert_eq!(encode_varint(0), vec![0]);
        assert_eq!(encode_varint(1), vec![1]);
        assert_eq!(encode_varint(127), vec![127]);
        assert_eq!(encode_varint(128), vec![0x80, 0x01]);
        assert_eq!(encode_varint(300), vec![0xAC, 0x02]);
    }

    #[test]
    fn test_encode_string_field() {
        let result = encode_string_field(1, "test");
        assert_eq!(result, vec![0x0A, 4, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_empty_fields_omitted() {
        assert!(encode_varint_field(1, 0).is_empty());
        assert!(encode_string_field(1, "").is_empty());
        assert!(encode_bytes_field(1, &[]).is_empty());
    }
}
