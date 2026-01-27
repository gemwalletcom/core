use std::sync::Arc;
use std::time::Duration;

use gem_tracing::info_with_fields;
use tokio::sync::watch;

pub type ShutdownSender = Arc<watch::Sender<bool>>;
pub type ShutdownReceiver = watch::Receiver<bool>;

pub fn channel() -> (ShutdownSender, ShutdownReceiver) {
    let (tx, rx) = watch::channel(false);
    (Arc::new(tx), rx)
}

pub fn spawn_signal_handler(shutdown_tx: ShutdownSender) {
    tokio::spawn(async move {
        wait_for_signal().await;
        info_with_fields!("shutdown signal received", status = "ok");
        let _ = shutdown_tx.send(true);
    });
}

pub async fn wait_with_timeout(handles: Vec<tokio::task::JoinHandle<()>>, timeout: Duration) {
    let _ = tokio::time::timeout(timeout, futures::future::join_all(handles)).await;
}

async fn wait_for_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info_with_fields!("received SIGINT", status = "ok"),
        _ = terminate => info_with_fields!("received SIGTERM", status = "ok"),
    }
}
