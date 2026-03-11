use std::cmp::Ordering;

pub trait PrioritizedProvider {
    fn provider_id(&self) -> &str;
    fn priority(&self) -> Option<i32>;
    fn threshold_bps(&self) -> Option<i32>;
}

pub fn sort_by_priority_then_amount<P: PrioritizedProvider>(
    a_id: &str,
    b_id: &str,
    a_amount: f64,
    b_amount: f64,
    providers: &[P],
    ascending: bool,
) -> Ordering {
    let a_provider = providers.iter().find(|p| p.provider_id() == a_id);
    let b_provider = providers.iter().find(|p| p.provider_id() == b_id);
    let a_pri = a_provider.and_then(|p| p.priority());
    let b_pri = b_provider.and_then(|p| p.priority());

    let by_amount = || {
        let ord = a_amount.partial_cmp(&b_amount).unwrap_or(Ordering::Equal);
        if ascending { ord } else { ord.reverse() }
    };

    match (a_pri, b_pri) {
        (Some(a), Some(b)) if a != b => {
            let higher_pri = if a < b { a_provider } else { b_provider }.unwrap();
            if exceeds_threshold(higher_pri, a_amount, b_amount, ascending) {
                by_amount()
            } else {
                a.cmp(&b)
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        _ => by_amount(),
    }
}

fn exceeds_threshold<P: PrioritizedProvider>(provider: &P, a: f64, b: f64, ascending: bool) -> bool {
    let Some(bps) = provider.threshold_bps() else {
        return false;
    };
    let better = if ascending { a.min(b) } else { a.max(b) };
    let diff = (a - b).abs() / better;
    diff > bps as f64 / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        id: String,
        priority: Option<i32>,
        threshold_bps: Option<i32>,
    }

    impl MockProvider {
        fn new(id: &str, priority: Option<i32>, threshold_bps: Option<i32>) -> Self {
            Self {
                id: id.to_string(),
                priority,
                threshold_bps,
            }
        }
    }

    impl PrioritizedProvider for MockProvider {
        fn provider_id(&self) -> &str {
            &self.id
        }
        fn priority(&self) -> Option<i32> {
            self.priority
        }
        fn threshold_bps(&self) -> Option<i32> {
            self.threshold_bps
        }
    }

    #[test]
    fn test_no_priority_sorts_by_amount_desc() {
        let providers: Vec<MockProvider> = vec![];
        let result = sort_by_priority_then_amount("a", "b", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_priority_wins_over_amount() {
        let providers = vec![
            MockProvider::new("a", Some(1), None),
            MockProvider::new("b", Some(2), None),
        ];
        let result = sort_by_priority_then_amount("a", "b", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_threshold_override() {
        let providers = vec![
            MockProvider::new("a", Some(1), Some(500)),
            MockProvider::new("b", Some(2), None),
        ];
        // b has 100% more than a, exceeds 5% threshold → amount wins
        let result = sort_by_priority_then_amount("a", "b", 100.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Greater);
    }

    #[test]
    fn test_threshold_not_exceeded() {
        let providers = vec![
            MockProvider::new("a", Some(1), Some(5000)),
            MockProvider::new("b", Some(2), None),
        ];
        // b has 10% more than a, within 50% threshold → priority wins
        let result = sort_by_priority_then_amount("a", "b", 100.0, 110.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_unprioritized_sorted_after_prioritized() {
        let providers = vec![MockProvider::new("a", Some(1), None)];
        let result = sort_by_priority_then_amount("a", "b", 50.0, 200.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_ascending_order() {
        let providers: Vec<MockProvider> = vec![];
        let result = sort_by_priority_then_amount("a", "b", 100.0, 200.0, &providers, true);
        assert_eq!(result, Ordering::Less);
    }

    #[test]
    fn test_same_priority_sorts_by_amount() {
        let providers = vec![
            MockProvider::new("a", Some(1), None),
            MockProvider::new("b", Some(1), None),
        ];
        let result = sort_by_priority_then_amount("a", "b", 200.0, 100.0, &providers, false);
        assert_eq!(result, Ordering::Less);
    }
}
