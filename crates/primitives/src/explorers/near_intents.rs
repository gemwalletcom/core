use super::near::NEAR_BLOCKS_BASE_URL;
use crate::block_explorer::{BlockExplorer, ExplorerInput};

pub struct NearIntents;

impl NearIntents {
    const BASE_URL: &'static str = "https://explorer.near-intents.org";

    pub fn boxed() -> Box<dyn BlockExplorer> {
        Box::new(Self)
    }
}

impl BlockExplorer for NearIntents {
    fn name(&self) -> String {
        "NEAR Intents".to_string()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/transactions/{}", Self::BASE_URL, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{NEAR_BLOCKS_BASE_URL}/address/{address}")
    }

    fn get_swap_tx_url(&self, input: &ExplorerInput) -> String {
        let base = self.get_tx_url(input.recipient.as_deref().unwrap_or_default());
        if let Some(memo) = input.memo.as_deref() {
            format!("{base}?depositMemo={memo}")
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_swap_tx_url() {
        let recipient = "GDJ4JZXZELZD737NVFORH4PSSQDWFDZTKW3AIDKHYQG23ZXBPDGGQBJK";
        let base = format!("{}/transactions/{recipient}", NearIntents::BASE_URL);

        assert_eq!(NearIntents.get_swap_tx_url(&ExplorerInput::new_recipient(recipient)), base);
        assert_eq!(
            NearIntents.get_swap_tx_url(&ExplorerInput::new_memo(recipient, "48694126")),
            format!("{base}?depositMemo=48694126")
        );
    }
}
