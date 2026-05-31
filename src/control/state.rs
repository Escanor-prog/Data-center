use crate::control::fsm::FsmState;
use crate::scenarios::types::PhaseResultStatus;
use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};
use tokio::sync::oneshot;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PhaseResult {
    pub phase: u8,
    pub scenario: String,
    pub status: PhaseResultStatus,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct SimulationState {
    pub fsm: FsmState,
    pub active: bool,
    pub simulation_id: Option<String>,
    pub simulation_token: Option<String>,
    pub scenario: Option<String>,
    pub target: Option<String>,
    pub authorized_by: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub max_duration_secs: u64,
    pub phase_results: Vec<PhaseResult>,
    pub phase_ack_tx: Option<oneshot::Sender<()>>,
    pub cancel_requested: bool,
}

pub type SharedSimulationState = Arc<RwLock<SimulationState>>;

impl SimulationState {
    pub fn new() -> Self {
        Self {
            fsm: FsmState::Idle,
            active: false,
            simulation_id: None,
            simulation_token: None,
            scenario: None,
            target: None,
            authorized_by: None,
            started_at: None,
            max_duration_secs: 0,
            phase_results: Vec::new(),
            phase_ack_tx: None,
            cancel_requested: false,
        }
    }

    pub fn start(
        &mut self,
        simulation_id: String,
        simulation_token: String,
        scenario: String,
        target: String,
        authorized_by: Option<String>,
        max_duration_secs: u64,
    ) -> Result<(), &'static str> {
        if self.active {
            return Err("SIMULATION_ACTIVE");
        }

        self.fsm = FsmState::Validating;
        self.active = true;
        self.simulation_id = Some(simulation_id);
        self.simulation_token = Some(simulation_token);
        self.scenario = Some(scenario);
        self.target = Some(target);
        self.authorized_by = authorized_by;
        self.started_at = Some(Utc::now());
        self.max_duration_secs = max_duration_secs;
        self.phase_results.clear();
        self.cancel_requested = false;
        Ok(())
    }

    pub fn set_fsm(&mut self, fsm: FsmState) {
        self.fsm = fsm;
    }

    pub fn push_phase_result(&mut self, result: PhaseResult) {
        self.phase_results.push(result);
    }

    pub fn take_phase_ack_tx(&mut self) -> Option<oneshot::Sender<()>> {
        self.phase_ack_tx.take()
    }

    pub fn set_phase_ack_tx(&mut self, tx: oneshot::Sender<()>) {
        self.phase_ack_tx = Some(tx);
    }

    pub fn signal_phase_ack(&mut self) {
        if let Some(tx) = self.phase_ack_tx.take() {
            let _ = tx.send(());
        }
    }

    pub fn request_cancel(&mut self) {
        self.cancel_requested = true;
        self.signal_phase_ack();
    }

    pub fn stop(&mut self) {
        *self = SimulationState::new();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub fn new_shared_state() -> SharedSimulationState {
    Arc::new(RwLock::new(SimulationState::new()))
}
