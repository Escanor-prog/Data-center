use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct SimulationMetrics {
    pub simulations_total: AtomicU64,
    pub simulations_success: AtomicU64,
    pub simulations_error: AtomicU64,
    pub simulations_timeout: AtomicU64,
    pub simulations_stopped: AtomicU64,
    pub phase1_ok: AtomicU64,
    pub phase1_timeout: AtomicU64,
    pub phase2_ok: AtomicU64,
    pub phase2_timeout: AtomicU64,
    pub phase3_ok: AtomicU64,
    pub phase3_timeout: AtomicU64,
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub simulations_total: u64,
    pub simulations_success: u64,
    pub simulations_error: u64,
    pub simulations_timeout: u64,
    pub simulations_stopped: u64,
    pub phase1_ok: u64,
    pub phase1_timeout: u64,
    pub phase2_ok: u64,
    pub phase2_timeout: u64,
    pub phase3_ok: u64,
    pub phase3_timeout: u64,
}

impl SimulationMetrics {
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            simulations_total: self.simulations_total.load(Ordering::Relaxed),
            simulations_success: self.simulations_success.load(Ordering::Relaxed),
            simulations_error: self.simulations_error.load(Ordering::Relaxed),
            simulations_timeout: self.simulations_timeout.load(Ordering::Relaxed),
            simulations_stopped: self.simulations_stopped.load(Ordering::Relaxed),
            phase1_ok: self.phase1_ok.load(Ordering::Relaxed),
            phase1_timeout: self.phase1_timeout.load(Ordering::Relaxed),
            phase2_ok: self.phase2_ok.load(Ordering::Relaxed),
            phase2_timeout: self.phase2_timeout.load(Ordering::Relaxed),
            phase3_ok: self.phase3_ok.load(Ordering::Relaxed),
            phase3_timeout: self.phase3_timeout.load(Ordering::Relaxed),
        }
    }

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

    pub fn record_phase(&self, phase: u8, ok: bool) {
        let (ok_counter, timeout_counter) = match phase {
            1 => (&self.phase1_ok, &self.phase1_timeout),
            2 => (&self.phase2_ok, &self.phase2_timeout),
            3 => (&self.phase3_ok, &self.phase3_timeout),
            _ => return,
        };
        if ok {
            ok_counter.fetch_add(1, Ordering::Relaxed);
        } else {
            timeout_counter.fetch_add(1, Ordering::Relaxed);
        }
    }
}
