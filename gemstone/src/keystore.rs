use gem_keystore::Mnemonic;

#[uniffi::export]
pub fn suggest_recovery_phrase_words(prefix: &str) -> Vec<String> {
    Mnemonic::suggest(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_recovery_phrase_words() {
        assert_eq!(suggest_recovery_phrase_words("woo"), vec!["wood", "wool"]);
        assert_eq!(suggest_recovery_phrase_words("woof"), Vec::<String>::new());
    }
}
