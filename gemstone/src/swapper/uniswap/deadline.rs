use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_DEADLINE: u64 = 3600;

pub fn get_sig_deadline() -> u64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
    now + DEFAULT_DEADLINE
}
