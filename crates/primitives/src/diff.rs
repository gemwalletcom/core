use std::collections::HashSet;
use std::hash::Hash;

pub struct Diff;

#[derive(Debug, Clone)]
pub struct DiffResult<T> {
    pub different: Vec<T>,
    pub missing: Vec<T>,
}

impl Diff {
    pub fn compare<T: Clone + Eq + Hash>(source: Vec<T>, target: Vec<T>) -> DiffResult<T> {
        let source_set: HashSet<T> = source.iter().cloned().collect();
        let target_set: HashSet<T> = target.iter().cloned().collect();

        let different: Vec<T> = source_set.difference(&target_set).cloned().collect();
        let missing: Vec<T> = target_set.difference(&source_set).cloned().collect();

        DiffResult { different, missing }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_strings() {
        let source = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let target = vec!["b".to_string(), "c".to_string(), "d".to_string()];

        let result = Diff::compare(source, target);

        assert_eq!(result.different, vec!["a".to_string()]);
        assert_eq!(result.missing, vec!["d".to_string()]);
    }
}
