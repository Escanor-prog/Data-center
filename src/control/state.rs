use chrono::{DateTime, Utc};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct SimulationState {
    pub active: bool,
    pub simulation_id: Option<String>,
    pub scenario: Option<String>,
    pub target: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub max_duration_secs: u64,
}

pub type SharedSimulationState = Arc<RwLock<SimulationState>>;

impl SimulationState {
    pub fn new() -> Self {
        Self {
            active: false,
            simulation_id: None,
            scenario: None,
            target: None,
            started_at: None,
            max_duration_secs: 0,
        }
    }

    pub fn start(
        &mut self,
        simulation_id: String,
        scenario: String,
        target: String,
        max_duration_secs: u64,
    ) -> Result<(), &'static str> {
        if self.active {
            return Err("Une simulation est déjà en cours");
        }

        self.active = true;
        self.simulation_id = Some(simulation_id);
        self.scenario = Some(scenario);
        self.target = Some(target);
        self.started_at = Some(Utc::now());
        self.max_duration_secs = max_duration_secs;
        Ok(())
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
