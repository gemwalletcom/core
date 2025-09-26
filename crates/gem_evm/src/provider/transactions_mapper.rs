pub fn map_transaction_broadcast(data: &str) -> String {
    if data.starts_with("0x") { data.to_string() } else { format!("0x{}", data) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_transaction_broadcast_encode() {
        assert_eq!(map_transaction_broadcast("123"), "0x123");
        assert_eq!(map_transaction_broadcast("0x123"), "0x123");
    }
}
