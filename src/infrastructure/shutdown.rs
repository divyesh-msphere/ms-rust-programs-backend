use tokio::sync::watch;
use tracing::info;

/// Installs a SIGINT/SIGTERM listener and returns a receiver that flips to
/// `true` on shutdown. Clone it for each server that needs to observe it.
pub fn install_signal_handler() -> watch::Receiver<bool> {
    let (tx, rx) = watch::channel(false);
    tokio::spawn(async move {
        wait_for_signal().await;
        info!("shutdown signal received");
        let _ = tx.send(true);
    });
    rx
}

async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
    }
}
