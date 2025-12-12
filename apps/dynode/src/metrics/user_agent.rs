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
