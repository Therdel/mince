use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Default, Clone)]
pub struct StopToken {
    state: Arc<AtomicBool>,
}

impl StopToken {
    pub fn request_stop(&self) {
        self.state.store(true, Ordering::SeqCst)
    }

    pub fn stop_requested(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }
}
