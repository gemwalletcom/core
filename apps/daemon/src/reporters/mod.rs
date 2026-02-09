pub mod consumer;
pub mod parser;

use primitives::ReportedError;
use std::time::SystemTime;

fn record_error(errors: &mut Vec<ReportedError>, error: &str) {
    let timestamp = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let message = if error.len() > 200 { &error[..200] } else { error };

    if let Some(entry) = errors.iter_mut().find(|e| e.message == message) {
        entry.count += 1;
        entry.timestamp = timestamp;
    } else {
        errors.push(ReportedError {
            message: message.to_string(),
            count: 1,
            timestamp,
        });
    }
}
