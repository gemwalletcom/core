use bip39::Language;

pub struct Mnemonic;

impl Mnemonic {
    pub fn suggest(prefix: &str) -> Vec<String> {
        let prefix = &prefix.trim().to_lowercase();
        if prefix.is_empty() {
            return Vec::new();
        }
        Language::English.words_by_prefix(prefix).iter().map(|word| word.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest() {
        assert_eq!(Mnemonic::suggest("woo"), vec!["wood", "wool"]);
        assert_eq!(Mnemonic::suggest("abandon"), vec!["abandon"]);
        assert_eq!(Mnemonic::suggest("woof"), Vec::<String>::new());

        let all_words = Mnemonic::suggest(" ");
        assert_eq!(all_words.len(), 0);
    }
}
