use super::message::{MessageDecode, MessageEncode, MessageResult};

pub fn encode_grpc_frame(payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(5 + payload.len());
    body.push(0);
    body.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    body.extend_from_slice(payload);
    body
}

pub fn encode_grpc_message<M: MessageEncode>(message: &M) -> Vec<u8> {
    encode_grpc_frame(&message.encode())
}

pub fn decode_grpc_frame(body: &[u8]) -> MessageResult<&[u8]> {
    if body.len() < 5 {
        return Err("gRPC response is missing message frame".into());
    }
    if body[0] != 0 {
        return Err("compressed gRPC responses are not supported".into());
    }
    let len = u32::from_be_bytes(body[1..5].try_into()?) as usize;
    let end = 5usize.checked_add(len).ok_or("invalid gRPC response frame length")?;
    if body.len() < end {
        return Err("truncated gRPC response frame".into());
    }
    Ok(&body[5..end])
}

pub fn decode_grpc_message<M: MessageDecode>(body: &[u8]) -> MessageResult<M> {
    M::decode(decode_grpc_frame(body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protobuf::encode_string_field;

    #[test]
    fn test_grpc_frame() {
        let payload = encode_string_field(1, "test");
        let frame = encode_grpc_frame(&payload);

        assert_eq!(decode_grpc_frame(&frame).unwrap(), payload.as_slice());
    }
}
