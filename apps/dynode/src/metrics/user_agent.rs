use crate::config::UserAgentPatterns;
use regex::Regex;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct CompiledPattern {
    category: String,
    regex: Regex,
}

#[derive(Debug, Clone)]
pub struct UserAgentMatcher {
    patterns: Arc<[CompiledPattern]>,
}

impl UserAgentMatcher {
    pub fn new(config: &UserAgentPatterns) -> Self {
        let patterns: Arc<[CompiledPattern]> = config
            .patterns
            .iter()
            .flat_map(|(category, patterns)| {
                patterns.iter().filter_map(|pattern| {
                    Regex::new(pattern).ok().map(|regex| CompiledPattern {
                        category: category.clone(),
                        regex,
                    })
                })
            })
            .collect::<Vec<_>>()
            .into();

        Self { patterns }
    }

    pub fn categorize(&self, user_agent: &str) -> &str {
        for pattern in self.patterns.iter() {
            if pattern.regex.is_match(user_agent) {
                return &pattern.category;
            }
        }
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_matcher() -> UserAgentMatcher {
        let mut patterns = HashMap::new();
        patterns.insert("mobile".to_string(), vec!["Mobile/.*".to_string()]);
        patterns.insert("desktop".to_string(), vec!["Desktop/.*".to_string()]);
        UserAgentMatcher::new(&UserAgentPatterns { patterns })
    }

    #[test]
    fn test_categorize() {
        let matcher = create_matcher();
        assert_eq!(matcher.categorize("Mobile/1.0"), "mobile");
        assert_eq!(matcher.categorize("Desktop/2.0"), "desktop");
        assert_eq!(matcher.categorize("Other/1.0"), "unknown");
        assert_eq!(matcher.categorize(""), "unknown");
    }
}
