pub struct Diff;

#[derive(Debug, Clone)]
pub struct DiffResult {
    pub different: Vec<String>,
    pub missing: Vec<String>,
}

impl Diff {
    /// Compares two vectors of strings and returns the differences and missing elements.
    ///
    /// # Arguments
    ///
    /// * `source` - A vector of strings representing the source data.
    /// * `target` - A vector of strings representing the target data.
    ///
    /// # Returns
    ///
    /// A `DiffResult` struct containing:
    /// * `different` - A vector of strings that are in the source but not in the target.
    /// * `missing` - A vector of strings that are in the target but not in the source.
    ///
    /// # Examples
    ///
    /// ```
    /// let source = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    /// let target = vec!["b".to_string(), "c".to_string(), "d".to_string()];
    ///
    /// let result = Diff::compare(source, target);
    ///
    /// assert_eq!(result.different, vec!["a".to_string()]);
    /// assert_eq!(result.missing, vec!["d".to_string()]);
    /// ```
    pub fn compare(source: Vec<String>, target: Vec<String>) -> DiffResult {
        let source_set: std::collections::HashSet<_> = source.iter().cloned().collect();
        let target_set: std::collections::HashSet<_> = target.iter().cloned().collect();

        let different: Vec<String> = source_set.difference(&target_set).cloned().collect();
        let missing: Vec<String> = target_set.difference(&source_set).cloned().collect();

        DiffResult { different, missing }
    }
}
