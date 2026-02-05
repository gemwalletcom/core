use crate::shutdown::ShutdownReceiver;
use crate::worker::context::WorkerContext;
use job_runner::JobHandle;
use std::error::Error;

pub async fn jobs(_ctx: WorkerContext, _shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    Ok(vec![])
}
