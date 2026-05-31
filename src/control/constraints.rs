use crate::config::Config;
use crate::scenarios::catalogue::{find_scenario, max_duration_for};
use chrono::{Local, Timelike};
use std::net::IpAddr;
use std::str::FromStr;

/// Contexte de lancement — tous les champs nécessaires aux contraintes C1–C15.
#[derive(Debug, Clone)]
pub struct LaunchContext {
    pub scenario: String,
    pub target: String,
    pub authorized_by: Option<String>,
    pub authorization_ref: Option<String>,
    pub snapshot_id: Option<String>,
    pub rollback_plan_id: Option<String>,
    pub max_duration_seconds: Option<u64>,
    pub token_valid: bool,
    pub simulation_active: bool,
}

#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub code: String,
    pub constraint: &'static str,
    pub message: String,
}

impl ConstraintViolation {
    fn new(constraint: &'static str, code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            constraint,
            message: message.into(),
        }
    }
}

/// Valide les 15 contraintes du binôme 11 avant tout lancement.
pub fn validate_all(ctx: &LaunchContext, cfg: &Config) -> Result<(), ConstraintViolation> {
    // C9 — une seule simulation à la fois
    check_no_active_simulation(ctx.simulation_active)
        .map_err(|e| ConstraintViolation::new("C9", &e, "Simulation déjà active"))?;

    // C3 — authentification lanceur
    if !ctx.token_valid {
        return Err(ConstraintViolation::new(
            "C3",
            "INSUFFICIENT_PERMISSIONS",
            "Token X-Agent-Token invalide ou absent",
        ));
    }

    // C2 — autorisation formelle
    check_authorization(ctx, cfg)?;

    // C1 + C11 — isolation sandbox, pas de production
    check_sandbox_target(&ctx.target, cfg)?;

    // C10 — scénario connu + durée bornée
    check_scenario_and_duration(ctx)?;

    // C10 — fenêtre horaire
    check_launch_window(cfg)?;

    // C4 — snapshot (hors dry_run strict)
    check_snapshot(ctx, cfg)?;

    // C12 — plan de rollback
    check_rollback_plan(ctx, cfg)?;

    Ok(())
}

pub fn check_no_active_simulation(active: bool) -> Result<(), String> {
    if active {
        Err("SIMULATION_ACTIVE".to_string())
    } else {
        Ok(())
    }
}

pub fn check_token(received: &str, expected: &str) -> Result<(), String> {
    if received == expected {
        Ok(())
    } else {
        Err("INSUFFICIENT_PERMISSIONS".to_string())
    }
}

fn check_authorization(ctx: &LaunchContext, cfg: &Config) -> Result<(), ConstraintViolation> {
    if !cfg.require_authorization {
        return Ok(());
    }
    let author = ctx.authorized_by.as_deref().unwrap_or("").trim();
    if author.is_empty() {
        return Err(ConstraintViolation::new(
            "C2",
            "ADMIN_APPROVAL_REQUIRED",
            "authorized_by obligatoire (autorisation formelle C2)",
        ));
    }
    if cfg.require_authorization_ref {
        let ref_id = ctx.authorization_ref.as_deref().unwrap_or("").trim();
        if ref_id.is_empty() {
            return Err(ConstraintViolation::new(
                "C2",
                "INVALID_REQUEST",
                "authorization_ref obligatoire (référence écrite C2)",
            ));
        }
    }
    Ok(())
}

fn check_sandbox_target(target: &str, cfg: &Config) -> Result<(), ConstraintViolation> {
    let ip = IpAddr::from_str(target.trim()).map_err(|_| {
        ConstraintViolation::new("C1", "INVALID_IP_FORMAT", format!("IP invalide: {target}"))
    })?;

    if !ip_in_cidr(ip, &cfg.sandbox_cidr) {
        return Err(ConstraintViolation::new(
            "C1",
            "INVALID_REQUEST",
            format!("Cible {target} hors sandbox {} (C1)", cfg.sandbox_cidr),
        ));
    }

    for blocked in &cfg.production_cidrs {
        if ip_in_cidr(ip, blocked) {
            return Err(ConstraintViolation::new(
                "C11",
                "INVALID_REQUEST",
                format!("Cible {target} dans segment production {blocked} (C11)"),
            ));
        }
    }

    Ok(())
}

fn check_scenario_and_duration(ctx: &LaunchContext) -> Result<(), ConstraintViolation> {
    find_scenario(&ctx.scenario).ok_or_else(|| {
        ConstraintViolation::new(
            "C10",
            "INVALID_REQUEST",
            format!("Scénario inconnu: {}", ctx.scenario),
        )
    })?;

    let max_allowed = max_duration_for(&ctx.scenario);
    if let Some(req) = ctx.max_duration_seconds {
        if req > max_allowed {
            return Err(ConstraintViolation::new(
                "C10",
                "INVALID_REQUEST",
                format!(
                    "Durée demandée {req}s > max {max_allowed}s pour {} (C10)",
                    ctx.scenario
                ),
            ));
        }
    }
    Ok(())
}

fn check_launch_window(cfg: &Config) -> Result<(), ConstraintViolation> {
    if !cfg.enforce_launch_window {
        return Ok(());
    }
    let hour = Local::now().hour();
    if hour >= cfg.launch_hour_start && hour < cfg.launch_hour_end {
        Ok(())
    } else {
        Err(ConstraintViolation::new(
            "C10",
            "INVALID_REQUEST",
            format!(
                "Hors fenêtre autorisée {start}h-{end}h (C10)",
                start = cfg.launch_hour_start,
                end = cfg.launch_hour_end
            ),
        ))
    }
}

fn check_snapshot(ctx: &LaunchContext, cfg: &Config) -> Result<(), ConstraintViolation> {
    if cfg.dry_run || !cfg.require_snapshot {
        return Ok(());
    }
    let snap = ctx.snapshot_id.as_deref().unwrap_or("").trim();
    if snap.is_empty() {
        Err(ConstraintViolation::new(
            "C4",
            "INVALID_REQUEST",
            "snapshot_id obligatoire avant lancement (C4 — Versionneur)",
        ))
    } else {
        Ok(())
    }
}

fn check_rollback_plan(ctx: &LaunchContext, cfg: &Config) -> Result<(), ConstraintViolation> {
    if cfg.dry_run || !cfg.require_rollback_plan {
        return Ok(());
    }
    let plan = ctx.rollback_plan_id.as_deref().unwrap_or("").trim();
    if plan.is_empty() {
        Err(ConstraintViolation::new(
            "C12",
            "INVALID_REQUEST",
            "rollback_plan_id obligatoire (C12 — plan retour état normal)",
        ))
    } else {
        Ok(())
    }
}

pub fn ip_in_cidr(ip: IpAddr, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return true;
    }
    let Ok(network) = IpAddr::from_str(parts[0]) else {
        return false;
    };
    let Ok(prefix) = parts[1].parse::<u8>() else {
        return false;
    };
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            let mask = if prefix >= 32 {
                u32::MAX
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(ip) & mask) == (u32::from(net) & mask)
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c9_blocks_active() {
        assert!(check_no_active_simulation(true).is_err());
    }

    #[test]
    fn c1_sandbox() {
        let cfg = Config {
            agent_token: "t".into(),
            decideur_token: None,
            auditeur_url: "".into(),
            decideur_url: "".into(),
            analyseur_url: "".into(),
            chiffreur_url: "".into(),
            trusted_ca: None,
            simulateur_whitelist: vec![],
            sandbox_cidr: "192.168.99.0/24".into(),
            production_cidrs: vec!["10.0.0.0/8".into()],
            dry_run: true,
            detection_timeout_secs: 60,
            phase1_scenario: "ddos_syn".into(),
            phase2_scenario: "brute_force".into(),
            phase3_scenario: "mot_de_passe_faible".into(),
            min_phases_ok_for_success: 2,
            backup_events_path: "var/log".into(),
            listen_port: 8005,
            require_authorization: true,
            require_authorization_ref: false,
            require_snapshot: false,
            require_rollback_plan: false,
            enforce_launch_window: false,
            enforce_health_checks: false,
            launch_hour_start: 8,
            launch_hour_end: 22,
            global_max_duration_secs: 900,
        };
        let ctx = LaunchContext {
            scenario: "ddos_syn".into(),
            target: "192.168.99.5".into(),
            authorized_by: Some("admin".into()),
            authorization_ref: None,
            snapshot_id: None,
            rollback_plan_id: None,
            max_duration_seconds: None,
            token_valid: true,
            simulation_active: false,
        };
        assert!(validate_all(&ctx, &cfg).is_ok());
    }
}
