#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmState {
    Idle,
    Validating,
    Handshake,
    Phase1Filtreur,
    Phase2Analyseur,
    Phase3Chiffreur,
    Reporting,
    Stopping,
    Failed,
}

impl FsmState {
    pub fn is_busy(self) -> bool {
        !matches!(self, FsmState::Idle | FsmState::Failed)
    }
}
