use primitives::unix_timestamp;

const DEFAULT_DEADLINE: u64 = 3600;

pub fn get_sig_deadline() -> u64 {
    unix_timestamp() + DEFAULT_DEADLINE
}
