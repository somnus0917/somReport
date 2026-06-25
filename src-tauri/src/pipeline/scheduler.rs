use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tauri::{AppHandle, Emitter};
use tokio::sync::watch;

use crate::domain::CaptureProvider;
use crate::pipeline::queue::QueueWorker;
use crate::providers;
use crate::storage::Database;
use crate::domain::VisionProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

#[derive(Clone)]
pub struct CaptureScheduler {
    state_tx: watch::Sender<RecordingState>,
    state_rx: watch::Receiver<RecordingState>,
    interval_sec: Arc<AtomicU64>,
}

impl CaptureScheduler {
    pub fn new(interval_sec: u64, _idle_threshold_sec: u64) -> Self {
        let (state_tx, state_rx) = watch::channel(RecordingState::Stopped);
        Self {
            state_tx,
            state_rx,
            interval_sec: Arc::new(AtomicU64::new(interval_sec.max(5))),
        }
    }

    pub fn state(&self) -> RecordingState {
        *self.state_rx.borrow()
    }

    pub fn state_rx(&self) -> watch::Receiver<RecordingState> {
        self.state_rx.clone()
    }

    pub fn start(&self) {
        self.state_tx.send_replace(RecordingState::Recording);
    }

    pub fn pause(&self) {
        self.state_tx.send_replace(RecordingState::Paused);
    }

    pub fn stop(&self) {
        self.state_tx.send_replace(RecordingState::Stopped);
    }

    pub fn interval_sec(&self) -> u64 {
        self.interval_sec.load(Ordering::Relaxed)
    }

    pub fn set_interval(&self, sec: u64) {
        self.interval_sec.store(sec.max(5), Ordering::Relaxed);
    }

    pub async fn run(
        &self,
        app: AppHandle,
        capture: Box<dyn CaptureProvider>,
        mut queue_worker: QueueWorker,
        db: Database,
        mut idle_rx: watch::Receiver<bool>,
    ) {
        let mut state_rx = self.state_rx.clone();
        let mut cached_provider: Option<Arc<dyn VisionProvider>> = None;
        let mut cached_provider_name = String::new();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.interval_sec())) => {
                    if *state_rx.borrow_and_update() != RecordingState::Recording {
                        continue;
                    }

                    if *idle_rx.borrow_and_update() {
                        log::debug!("User idle, skipping capture");
                        continue;
                    }

                    let settings = match db.get_settings() {
                        Ok(settings) => settings,
                        Err(error) => {
                            log::error!("Failed to read settings: {error}");
                            continue;
                        }
                    };

                    match db.get_daily_usage_cents(chrono::Local::now().date_naive()) {
                        Ok(cost) if cost >= f64::from(settings.max_daily_cost_cents) => {
                            log::warn!("Daily API budget reached; skipping capture");
                            continue;
                        }
                        Err(error) => log::warn!("Failed to read daily usage: {error}"),
                        _ => {}
                    }

                    let frame = match capture.capture().await {
                        Ok(Some(frame)) => frame,
                        Ok(None) => continue,
                        Err(error) => {
                            log::error!("Capture failed: {error}");
                            continue;
                        }
                    };

                    if cached_provider.is_none() || cached_provider_name != settings.vision_provider.name {
                        match providers::create_vision_provider(&settings.vision_provider) {
                            Ok(provider) => {
                                cached_provider = Some(provider);
                                cached_provider_name = settings.vision_provider.name.clone();
                            }
                            Err(error) => {
                                log::error!("Vision provider is unavailable: {error}");
                                continue;
                            }
                        }
                    }

                    let provider = cached_provider.as_ref().unwrap();

                    match queue_worker.process_frame(
                        &frame,
                        &**provider,
                        &settings.vision_provider.name,
                        &settings.vision_provider.model,
                        settings.vision_provider.input_cost_per_million_cents,
                        settings.vision_provider.output_cost_per_million_cents,
                        self.interval_sec(),
                    ).await {
                        Ok(activities) => {
                            for activity in activities {
                                let _ = app.emit("activity-created", activity);
                            }
                        }
                        Err(error) => log::error!("Frame processing failed: {error}"),
                    }
                }
                changed = state_rx.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    log::info!("Recording state changed to {:?}", *state_rx.borrow_and_update());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_stopped() {
        let scheduler = CaptureScheduler::new(5, 60);
        assert_eq!(scheduler.state(), RecordingState::Stopped);
    }

    #[test]
    fn state_transitions_are_observable() {
        let scheduler = CaptureScheduler::new(5, 60);
        let mut receiver = scheduler.state_rx();
        scheduler.start();
        assert_eq!(*receiver.borrow_and_update(), RecordingState::Recording);
        scheduler.pause();
        assert_eq!(*receiver.borrow_and_update(), RecordingState::Paused);
        scheduler.stop();
        assert_eq!(*receiver.borrow_and_update(), RecordingState::Stopped);
    }

    #[test]
    fn interval_is_clamped_and_can_be_updated() {
        let scheduler = CaptureScheduler::new(1, 60);
        assert_eq!(scheduler.interval_sec(), 5);
        scheduler.set_interval(15);
        assert_eq!(scheduler.interval_sec(), 15);
    }
}
