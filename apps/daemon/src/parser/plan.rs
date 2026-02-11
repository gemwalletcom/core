use std::cmp;
use std::time::Duration;

use chrono::Utc;
use storage::models::ParserStateRow;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlockPlanKind {
    Enqueue,
    Parse,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockRange {
    pub blocks: Vec<u64>,
    pub end_block: i64,
    pub remaining: i64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockPlan {
    pub range: BlockRange,
    pub kind: BlockPlanKind,
}

pub fn timeout_for_state(state: &ParserStateRow, min_check: Duration, max_check: Duration) -> Duration {
    let block_time = Duration::from_millis(state.block_time as u64);
    if block_time.is_zero() {
        return cmp::max(Duration::from_millis(state.timeout_latest_block as u64), min_check);
    }

    let elapsed = Utc::now().naive_utc().signed_duration_since(state.updated_at).num_milliseconds().max(0) as u64;

    let remaining = block_time.saturating_sub(Duration::from_millis(elapsed));
    let upper = cmp::max(cmp::min(block_time, max_check), min_check);
    remaining.clamp(min_check, upper)
}

pub fn should_reload_catchup(remaining: i64, interval: i64) -> bool {
    interval > 0 && remaining % interval == 0
}

pub fn plan_next_block(state: &ParserStateRow, current_block: i64, latest_block: i64) -> Option<BlockPlan> {
    let start_block = current_block + 1;
    let end_block = cmp::min(start_block + state.parallel_blocks as i64 - 1, latest_block - state.await_blocks as i64);
    if end_block < start_block {
        return None;
    }
    let blocks = (start_block..=end_block).map(|b| b as u64).collect::<Vec<_>>();
    let remaining = latest_block - end_block - state.await_blocks as i64;
    let kind = if let Some(queue_behind_blocks) = state.queue_behind_blocks
        && remaining > queue_behind_blocks as i64
    {
        BlockPlanKind::Enqueue
    } else {
        BlockPlanKind::Parse
    };

    Some(BlockPlan {
        range: BlockRange { blocks, end_block, remaining },
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::{BlockPlanKind, plan_next_block, should_reload_catchup, timeout_for_state};
    use chrono::Utc;
    use std::time::Duration;
    use storage::models::ParserStateRow;

    fn state(await_blocks: i32, parallel_blocks: i32, timeout_latest_block: i32, queue_behind_blocks: Option<i32>) -> ParserStateRow {
        ParserStateRow {
            chain: "ethereum".to_string(),
            current_block: 0,
            latest_block: 0,
            await_blocks,
            timeout_between_blocks: 0,
            timeout_latest_block,
            parallel_blocks,
            is_enabled: true,
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            queue_behind_blocks,
            block_time: 0,
        }
    }

    const MIN: Duration = Duration::from_secs(1);
    const MAX: Duration = Duration::from_secs(8);

    #[test]
    fn test_timeout_for_state_no_block_time() {
        let s = state(1, 1, 500, None);
        assert_eq!(timeout_for_state(&s, MIN, MAX), MIN);
        assert_eq!(timeout_for_state(&s, Duration::from_millis(100), MAX), Duration::from_millis(500));
    }

    #[test]
    fn test_timeout_for_state_uses_remaining_block_time() {
        let mut s = state(1, 1, 0, None);
        s.block_time = 12_000;
        s.updated_at = Utc::now().naive_utc() - chrono::Duration::seconds(4);
        let timeout = timeout_for_state(&s, MIN, MAX);
        assert!(timeout >= Duration::from_secs(7) && timeout <= Duration::from_secs(9));
    }

    #[test]
    fn test_timeout_for_state_caps_at_max() {
        let mut s = state(1, 1, 0, None);
        s.block_time = 600_000;
        s.updated_at = Utc::now().naive_utc();
        assert_eq!(timeout_for_state(&s, MIN, MAX), MAX);
    }

    #[test]
    fn test_timeout_for_state_overdue_block() {
        let mut s = state(1, 1, 0, None);
        s.block_time = 10_000;
        s.updated_at = Utc::now().naive_utc() - chrono::Duration::seconds(15);
        assert_eq!(timeout_for_state(&s, MIN, MAX), MIN);
    }

    #[test]
    fn test_should_reload_catchup_respects_interval() {
        assert!(!should_reload_catchup(10, 0));
        assert!(should_reload_catchup(10, 5));
        assert!(!should_reload_catchup(11, 5));
    }

    #[test]
    fn test_plan_next_block_returns_none_when_no_blocks() {
        let state = state(5, 3, 0, None);
        let plan = plan_next_block(&state, 10, 12);
        assert!(plan.is_none());
    }

    #[test]
    fn test_plan_next_block_builds_expected_blocks() {
        let state = state(1, 3, 0, None);
        let plan = plan_next_block(&state, 5, 10).unwrap();
        assert_eq!(plan.range.blocks, vec![6, 7, 8]);
        assert_eq!(plan.range.end_block, 8);
        assert_eq!(plan.range.remaining, 1);
        if let BlockPlanKind::Parse = plan.kind {
        } else {
            panic!("expected parse plan");
        }
    }

    #[test]
    fn test_plan_next_block_enqueues_when_behind() {
        let state = state(1, 3, 0, Some(2));
        let plan = plan_next_block(&state, 5, 20).unwrap();
        if let BlockPlanKind::Enqueue = plan.kind {
        } else {
            panic!("expected enqueue plan");
        }
    }
}
