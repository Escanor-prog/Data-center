use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct SimulationMetrics {
    pub simulations_total: AtomicU64,
    pub simulations_success: AtomicU64,
    pub simulations_error: AtomicU64,
    pub simulations_timeout: AtomicU64,
    pub simulations_stopped: AtomicU64,
}

impl SimulationMetrics {
    pub fn inc_total(&self) {
        self.simulations_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_success(&self) {
        self.simulations_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_error(&self) {
        self.simulations_error.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_timeout(&self) {
        self.simulations_timeout.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_stopped(&self) {
        self.simulations_stopped.fetch_add(1, Ordering::Relaxed);
    }
}
