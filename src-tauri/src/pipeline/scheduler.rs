use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Stopped,
    Recording,
    Paused,
}

pub struct CaptureScheduler {
    state_tx: watch::Sender<RecordingState>,
    state_rx: watch::Receiver<RecordingState>,
    interval_sec: u64,
    idle_threshold_sec: u64,
}

impl CaptureScheduler {
    pub fn new(interval_sec: u64, idle_threshold_sec: u64) -> Self {
        let (state_tx, state_rx) = watch::channel(RecordingState::Stopped);
        Self {
            state_tx,
            state_rx,
            interval_sec,
            idle_threshold_sec,
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
        self.interval_sec
    }

    pub fn idle_threshold_sec(&self) -> u64 {
        self.idle_threshold_sec
    }

    pub fn set_interval(&mut self, sec: u64) {
        self.interval_sec = sec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_stopped() {
        let s = CaptureScheduler::new(5, 60);
        assert_eq!(s.state(), RecordingState::Stopped);
    }

    #[test]
    fn start_transitions_to_recording() {
        let s = CaptureScheduler::new(5, 60);
        s.start();
        assert_eq!(s.state(), RecordingState::Recording);
    }

    #[test]
    fn pause_transitions_to_paused() {
        let s = CaptureScheduler::new(5, 60);
        s.start();
        s.pause();
        assert_eq!(s.state(), RecordingState::Paused);
    }

    #[test]
    fn stop_transitions_to_stopped() {
        let s = CaptureScheduler::new(5, 60);
        s.start();
        s.stop();
        assert_eq!(s.state(), RecordingState::Stopped);
    }

    #[test]
    fn stop_from_paused() {
        let s = CaptureScheduler::new(5, 60);
        s.start();
        s.pause();
        s.stop();
        assert_eq!(s.state(), RecordingState::Stopped);
    }

    #[test]
    fn start_from_paused_resumes() {
        let s = CaptureScheduler::new(5, 60);
        s.start();
        s.pause();
        s.start();
        assert_eq!(s.state(), RecordingState::Recording);
    }

    #[test]
    fn state_rx_receives_updates() {
        let s = CaptureScheduler::new(5, 60);
        let mut rx = s.state_rx();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Stopped);

        s.start();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Recording);

        s.pause();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Paused);

        s.stop();
        assert_eq!(*rx.borrow_and_update(), RecordingState::Stopped);
    }

    #[test]
    fn interval_sec_returns_configured_value() {
        let s = CaptureScheduler::new(10, 120);
        assert_eq!(s.interval_sec(), 10);
    }

    #[test]
    fn idle_threshold_sec_returns_configured_value() {
        let s = CaptureScheduler::new(5, 90);
        assert_eq!(s.idle_threshold_sec(), 90);
    }

    #[test]
    fn set_interval_updates_value() {
        let mut s = CaptureScheduler::new(5, 60);
        s.set_interval(15);
        assert_eq!(s.interval_sec(), 15);
    }
}
