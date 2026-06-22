use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{Duration, interval};

pub struct IdleDetector {
    threshold_sec: u64,
    idle_tx: watch::Sender<bool>,
    idle_rx: watch::Receiver<bool>,
}

impl IdleDetector {
    pub fn new(threshold_sec: u64) -> Self {
        let (idle_tx, idle_rx) = watch::channel(false);
        Self {
            threshold_sec,
            idle_tx,
            idle_rx,
        }
    }

    pub fn is_idle(&self) -> bool {
        *self.idle_rx.borrow()
    }

    pub fn idle_rx(&self) -> watch::Receiver<bool> {
        self.idle_rx.clone()
    }

    pub async fn run(self: Arc<Self>) {
        if self.try_dbus().await.is_ok() {
            return;
        }
        log::info!("D-Bus idle detection unavailable, falling back to polling");
        self.poll_fallback().await;
    }

    async fn try_dbus(&self) -> Result<(), Box<dyn std::error::Error>> {
        use futures_util::StreamExt;

        let conn = zbus::Connection::session().await?;
        let mut stream = zbus::MessageStream::from(&conn);

        while let Some(result) = stream.next().await {
            let msg = result?;
            let header = msg.header();
            let is_screensaver_signal = header.interface().map(|i| i.as_str() == "org.freedesktop.ScreenSaver").unwrap_or(false)
                && header.member().map(|m| m.as_str() == "ActiveChanged").unwrap_or(false);
            if is_screensaver_signal
            {
                if let Ok(active) = msg.body().deserialize::<bool>() {
                    self.idle_tx.send(active).ok();
                }
            }
        }

        Ok(())
    }

    async fn poll_fallback(&self) {
        let mut ticker = interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let idle_secs = get_idle_seconds().unwrap_or(0);
            let is_idle = idle_secs >= self.threshold_sec;
            if *self.idle_rx.borrow() != is_idle {
                self.idle_tx.send(is_idle).ok();
            }
        }
    }
}

fn get_idle_seconds() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("xprintidle").output().ok()?;
        let ms: u64 = String::from_utf8(output.stdout).ok()?.trim().parse().ok()?;
        Some(ms / 1000)
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_not_idle() {
        let detector = IdleDetector::new(300);
        assert!(!detector.is_idle());
    }
}
