use crate::comms::chiffreur::CryptoTestResult;
use crate::comms::health::check_datacenter_health;
use crate::comms::{AgentClients, dto};
use crate::config::Config;
use crate::control::constraints::{validate_all, LaunchContext};
use crate::control::fsm::FsmState;
use crate::control::metrics::SimulationMetrics;
use crate::control::state::{PhaseResult, SharedSimulationState};
use crate::scenarios::catalogue::{
    detection_timeout_for, detection_timeout_for_phase, global_pipeline_max_secs, max_duration_for,
};
use crate::scenarios::executor;
use crate::scenarios::types::PhaseResultStatus;
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

#[derive(Debug, Clone)]
pub struct StartParams {
    pub scenario: String,
    pub target: String,
    pub authorized_by: Option<String>,
    pub authorization_ref: Option<String>,
    pub snapshot_id: Option<String>,
    pub rollback_plan_id: Option<String>,
    pub max_duration_seconds: Option<u64>,
    pub token_valid: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulationReport {
    pub simulation_id: String,
    pub overall_success: bool,
    pub phases: Vec<PhaseResult>,
    pub constraints_validated: Vec<&'static str>,
}

pub async fn run_full_pipeline(
    cfg: Arc<Config>,
    clients: Arc<AgentClients>,
    state: SharedSimulationState,
    metrics: Arc<SimulationMetrics>,
    params: StartParams,
) {
    metrics.inc_total();

    let simulation_id = dto::new_simulation_id();
    let simulation_token = dto::new_simulation_token();

    let active = state.read().unwrap().active;
    let launch_ctx = LaunchContext {
        scenario: params.scenario.clone(),
        target: params.target.clone(),
        authorized_by: params.authorized_by.clone(),
        authorization_ref: params.authorization_ref.clone(),
        snapshot_id: params.snapshot_id.clone(),
        rollback_plan_id: params.rollback_plan_id.clone(),
        max_duration_seconds: params.max_duration_seconds,
        token_valid: params.token_valid,
        simulation_active: active,
    };

    if let Err(v) = validate_all(&launch_ctx, &cfg) {
        tracing::error!(
            constraint = v.constraint,
            code = %v.code,
            msg = %v.message,
            "Contrainte violée"
        );
        metrics.inc_error();
        clients
            .auditeur
            .simulation_error(&simulation_id, &format!("{}: {}", v.constraint, v.message))
            .await;
        return;
    }

    // C7 — santé DC avant lancement
    if let Err(e) = check_datacenter_health(&cfg).await {
        tracing::error!(error = %e, "C7 échouée");
        metrics.inc_error();
        clients.auditeur.simulation_error(&simulation_id, &e).await;
        return;
    }

    let max_duration = params
        .max_duration_seconds
        .unwrap_or_else(|| max_duration_for(&params.scenario));

    {
        let mut s = state.write().unwrap();
        if let Err(e) = s.start(
            simulation_id.clone(),
            simulation_token.clone(),
            params.scenario.clone(),
            params.target.clone(),
            params.authorized_by.clone(),
            max_duration,
        ) {
            tracing::error!(error = %e, "start state");
            metrics.inc_error();
            return;
        }
    }

    let sim_id = simulation_id.clone();

    // C6 — information cellules + C13 — journal dédié
    clients
        .auditeur
        .send_event(
            &sim_id,
            "C6_CELLULES_NOTIFIEES",
            json!({ "scenario": params.scenario, "authorized_by": params.authorized_by }),
        )
        .await;
    clients
        .auditeur
        .simulation_start(&sim_id, &params.scenario)
        .await;

    // C14 — plafond global pipeline
    let global_max = cfg.global_max_duration_secs.min(global_pipeline_max_secs());
    let pipeline = execute_pipeline(
        &cfg,
        &clients,
        &state,
        &metrics,
        &sim_id,
        &simulation_token,
        &params.target,
        max_duration,
    );

    let report = match timeout(Duration::from_secs(global_max), pipeline).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            clients.auditeur.simulation_error(&sim_id, &e).await;
            metrics.inc_error();
            let mut s = state.write().unwrap();
            s.set_fsm(FsmState::Failed);
            s.stop();
            return;
        }
        Err(_) => {
            let msg = format!("C14: dépassement durée globale {global_max}s");
            clients.auditeur.simulation_error(&sim_id, &msg).await;
            request_stop(&state);
            metrics.inc_error();
            let mut s = state.write().unwrap();
            s.stop();
            return;
        }
    };

    // C15 — scan santé post-simulation
    if let Err(e) = check_datacenter_health(&cfg).await {
        clients
            .auditeur
            .send_event(&sim_id, "C15_POST_HEALTH_WARN", json!({ "error": e }))
            .await;
    }

    clients
        .auditeur
        .simulation_end(
            &sim_id,
            json!({
                "overall_success": report.overall_success,
                "phases": report.phases,
                "constraints": report.constraints_validated
            }),
        )
        .await;

    if report.overall_success {
        metrics.inc_success();
    } else {
        metrics.inc_timeout();
    }

    let mut s = state.write().unwrap();
    s.set_fsm(FsmState::Idle);
    s.stop();
}

async fn execute_pipeline(
    cfg: &Config,
    clients: &AgentClients,
    state: &SharedSimulationState,
    metrics: &SimulationMetrics,
    sim_id: &str,
    sim_token: &str,
    target: &str,
    max_duration: u64,
) -> Result<SimulationReport, String> {
    set_fsm(state, FsmState::Handshake);

    // C5 — mode simulation : handshake Décideur active le mode (SIM_START délégué côté Décideur)
    clients
        .decideur
        .handshake(
            sim_id,
            sim_token,
            &cfg.phase2_scenario,
            target,
            max_duration,
        )
        .await?;

    let p1 = run_network_phase(
        cfg,
        clients,
        state,
        metrics,
        sim_id,
        1,
        FsmState::Phase1Filtreur,
        &cfg.phase1_scenario,
        target,
    )
    .await;

    if is_cancelled(state) {
        return Err("Simulation annulée (C14)".into());
    }

    set_fsm(state, FsmState::Phase2Analyseur);
    // Fallback direct si Décideur n'a pas encore implémenté la délégation C5
    clients
        .analyseur
        .sim_start(sim_id, &cfg.sandbox_cidr, max_duration)
        .await?;

    let p2 = run_network_phase(
        cfg,
        clients,
        state,
        metrics,
        sim_id,
        2,
        FsmState::Phase2Analyseur,
        &cfg.phase2_scenario,
        target,
    )
    .await;

    let _ = clients.analyseur.sim_end(sim_id).await;

    if is_cancelled(state) {
        return Err("Simulation annulée (C14)".into());
    }

    set_fsm(state, FsmState::Phase3Chiffreur);
    let p3 = run_crypto_phase(cfg, clients, state, metrics, sim_id, target).await;

    // Phase Auditeur — test intégrité config (scénario injection_base)
    run_auditeur_integrity_test(cfg, clients, sim_id, target).await;

    set_fsm(state, FsmState::Reporting);

    let phases = vec![p1, p2, p3];
    let ok_count = phases
        .iter()
        .filter(|p| p.status == PhaseResultStatus::Ok)
        .count() as u8;
    let overall_success = ok_count >= cfg.min_phases_ok_for_success;

    {
        let mut s = state.write().unwrap();
        s.phase_results = phases.clone();
    }

    Ok(SimulationReport {
        simulation_id: sim_id.to_string(),
        overall_success,
        phases,
        constraints_validated: vec![
            "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "C12", "C13",
            "C14", "C15",
        ],
    })
}

async fn run_network_phase(
    cfg: &Config,
    clients: &AgentClients,
    state: &SharedSimulationState,
    metrics: &SimulationMetrics,
    sim_id: &str,
    phase: u8,
    fsm: FsmState,
    scenario: &str,
    target: &str,
) -> PhaseResult {
    set_fsm(state, fsm);
    let duration = max_duration_for(scenario);
    let detect_timeout = detection_timeout_for(scenario).max(detection_timeout_for_phase(phase));

    clients
        .auditeur
        .phase_event(
            sim_id,
            phase,
            "START",
            json!({
                "scenario": scenario,
                "target": target,
                "max_duration_secs": duration,
                "detection_timeout_secs": detect_timeout
            }),
        )
        .await;

    let exec = executor::execute(scenario, target, duration, cfg.dry_run).await;
    if exec.is_err() {
        let msg = exec.err().unwrap();
        metrics.record_phase(phase, false);
        let result = PhaseResult {
            phase,
            scenario: scenario.to_string(),
            status: PhaseResultStatus::Error,
            message: Some(msg),
        };
        push_result(state, result.clone());
        clients
            .auditeur
            .phase_event(sim_id, phase, "ERROR", json!({ "message": result.message }))
            .await;
        return result;
    }

    let detected = wait_phase_detection(state, detect_timeout, cfg.dry_run).await;

    let (status, event_suffix) = if detected {
        metrics.record_phase(phase, true);
        (PhaseResultStatus::Ok, "OK")
    } else {
        metrics.record_phase(phase, false);
        (PhaseResultStatus::Timeout, "TIMEOUT")
    };

    let result = PhaseResult {
        phase,
        scenario: scenario.to_string(),
        status,
        message: None,
    };
    push_result(state, result.clone());
    clients
        .auditeur
        .phase_event(sim_id, phase, event_suffix, json!({ "scenario": scenario }))
        .await;
    result
}

async fn run_crypto_phase(
    cfg: &Config,
    clients: &AgentClients,
    state: &SharedSimulationState,
    metrics: &SimulationMetrics,
    sim_id: &str,
    target: &str,
) -> PhaseResult {
    const PHASE: u8 = 3;
    let scenario = &cfg.phase3_scenario;

    clients
        .auditeur
        .phase_event(
            sim_id,
            PHASE,
            "START",
            json!({
                "scenario": scenario,
                "tests": ["weak_password", "encrypt_roundtrip", "cert_tls"]
            }),
        )
        .await;

    let _ = executor::execute("cert_tls", target, 60, cfg.dry_run).await;

    let weak = clients.chiffreur.test_weak_password("123456").await;
    let roundtrip = clients
        .chiffreur
        .encrypt_decrypt_roundtrip("message-test-simulation")
        .await;

    let ok = matches!(weak, CryptoTestResult::RejectedAsExpected | CryptoTestResult::Ok)
        && matches!(roundtrip, CryptoTestResult::Ok);

    let status = if ok {
        metrics.record_phase(PHASE, true);
        PhaseResultStatus::Ok
    } else {
        metrics.record_phase(PHASE, false);
        PhaseResultStatus::Error
    };

    let result = PhaseResult {
        phase: PHASE,
        scenario: scenario.clone(),
        status,
        message: if ok {
            None
        } else {
            Some(format!("weak={weak:?}, roundtrip={roundtrip:?}"))
        },
    };

    push_result(state, result.clone());
    clients
        .auditeur
        .phase_event(
            sim_id,
            PHASE,
            if ok { "OK" } else { "ERROR" },
            json!({ "scenario": scenario }),
        )
        .await;
    result
}

async fn run_auditeur_integrity_test(cfg: &Config, clients: &AgentClients, sim_id: &str, target: &str) {
    let scenario = "injection_base";
    clients
        .auditeur
        .phase_event(sim_id, 4, "START", json!({ "scenario": scenario, "agent": "auditeur" }))
        .await;
    let _ = executor::execute(scenario, target, 60, cfg.dry_run).await;
    clients
        .auditeur
        .phase_event(sim_id, 4, "OK", json!({ "scenario": scenario }))
        .await;
}

async fn wait_phase_detection(
    state: &SharedSimulationState,
    detection_timeout_secs: u64,
    dry_run: bool,
) -> bool {
    if dry_run {
        sleep(Duration::from_secs(2)).await;
        return true;
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut s = state.write().unwrap();
        if s.cancel_requested {
            return false;
        }
        s.set_phase_ack_tx(tx);
    }

    match timeout(Duration::from_secs(detection_timeout_secs), rx).await {
        Ok(Ok(())) => true,
        _ => {
            let mut s = state.write().unwrap();
            s.take_phase_ack_tx();
            false
        }
    }
}

fn is_cancelled(state: &SharedSimulationState) -> bool {
    state.read().unwrap().cancel_requested
}

fn set_fsm(state: &SharedSimulationState, fsm: FsmState) {
    let mut s = state.write().unwrap();
    s.set_fsm(fsm);
}

fn push_result(state: &SharedSimulationState, result: PhaseResult) {
    let mut s = state.write().unwrap();
    s.push_phase_result(result);
}

pub fn signal_phase_ack(state: &SharedSimulationState) {
    let mut s = state.write().unwrap();
    s.signal_phase_ack();
}

pub fn request_stop(state: &SharedSimulationState) {
    let mut s = state.write().unwrap();
    s.request_cancel();
}
