//! Base de connaissances — 24 scénarios d'attaque (data center ENSPY).
//! Source : `knowledge/attacks_catalog.json` + Phase 0 (ANO-xxx).

use crate::scenarios::types::PhaseKind;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioSpec {
    pub id: &'static str,
    pub description: &'static str,
    pub agent_cible: &'static str,
    pub phase: PhaseKind,
    pub anomaly_ref: &'static str,
    pub mitre: &'static str,
    pub tool: &'static str,
    pub max_duration_secs: u64,
    pub detection_timeout_secs: u64,
    pub pipeline_default: bool,
}

static CATALOG: OnceLock<Vec<ScenarioSpec>> = OnceLock::new();

fn build_catalog() -> Vec<ScenarioSpec> {
    vec![
        // ── Filtreur (7) — trafic Internet ANO-016..022 ──
        spec("ddos_syn_ext", "SYN flood entrant Internet", "filtreur", PhaseKind::Filtreur, "ANO-018", "T1498", "hping3", 120, 45, true),
        spec("scan_ports_ext", "Scan ports externe", "filtreur", PhaseKind::Filtreur, "ANO-019", "T1046", "nmap", 180, 60, false),
        spec("brute_force_rdp_ext", "Brute force SSH/RDP externe", "filtreur", PhaseKind::Filtreur, "ANO-020", "T1110", "hydra", 300, 45, false),
        spec("icmp_flood_ext", "Ping flood ICMP externe", "filtreur", PhaseKind::Filtreur, "ANO-021", "T1498", "hping3", 120, 45, false),
        spec("port_interdit_ext", "Port interdit ingress", "filtreur", PhaseKind::Filtreur, "ANO-022", "T1046", "nc", 60, 45, false),
        spec("contact_cc_ext", "Contact C&C sortant", "filtreur", PhaseKind::Filtreur, "ANO-016", "T1071", "curl", 120, 45, false),
        spec("exfiltration_ext", "Exfiltration Internet", "filtreur", PhaseKind::Filtreur, "ANO-017", "T1048", "curl", 180, 60, false),
        // ── Analyseur (11) — trafic interne ANO-001..015 ──
        spec("ddos_syn_int", "SYN flood interne", "analyseur", PhaseKind::Analyseur, "ANO-001", "T1498", "hping3", 120, 90, false),
        spec("scan_ports_int", "Scan ports interne", "analyseur", PhaseKind::Analyseur, "ANO-002", "T1046", "nmap", 180, 90, false),
        spec("dns_tunnel", "Tunneling DNS", "analyseur", PhaseKind::Analyseur, "ANO-003", "T1071.004", "dig", 300, 90, false),
        spec("brute_force_ssh_int", "Brute force SSH inter-VM", "analyseur", PhaseKind::Analyseur, "ANO-004", "T1110", "hydra", 300, 90, true),
        spec("brute_force_auth", "Brute force authentification", "analyseur", PhaseKind::Analyseur, "ANO-005", "T1110", "hydra", 300, 90, false),
        spec("sql_injection", "Injection SQL", "analyseur", PhaseKind::Analyseur, "ANO-009", "T1190", "curl", 120, 90, false),
        spec("web_scraping", "Scraping HTTP massif", "analyseur", PhaseKind::Analyseur, "ANO-010", "T1119", "curl", 300, 90, false),
        spec("vlan_violation", "Violation isolation VLAN", "analyseur", PhaseKind::Analyseur, "ANO-013", "T1021", "hping3", 120, 90, false),
        spec("bandwidth_saturation", "Saturation bande passante", "analyseur", PhaseKind::Analyseur, "ANO-012", "T1498", "iperf3", 300, 90, false),
        spec("ransomware_sim", "Comportement ransomware simulé", "analyseur", PhaseKind::Analyseur, "ANO-008", "T1486", "script", 180, 90, false),
        spec("lateral_movement", "Mouvement latéral SSH", "analyseur", PhaseKind::Analyseur, "ANO-004", "T1021.004", "ssh", 300, 90, false),
        // ── Chiffreur (4) ──
        spec("cert_tls_expired", "Certificat TLS expiré", "chiffreur", PhaseKind::Chiffreur, "PLB-005", "T1553", "openssl", 60, 30, false),
        spec("weak_password", "Mot de passe faible", "chiffreur", PhaseKind::Chiffreur, "ANO-005", "T1110", "api", 60, 30, true),
        spec("key_rotation", "Rotation clés/credentials", "chiffreur", PhaseKind::Chiffreur, "PLB-003", "T1552", "api", 120, 30, false),
        spec("encrypt_roundtrip", "Chiffrement/déchiffrement", "chiffreur", PhaseKind::Chiffreur, "API", "T1573", "api", 60, 30, false),
        // ── Auditeur (2) ──
        spec("injection_base", "Injection règle malformée", "auditeur", PhaseKind::Auditeur, "ANO-021", "T1565", "curl", 90, 45, false),
        spec("audit_log_flood", "Flood logs audit", "auditeur", PhaseKind::Auditeur, "ANO-022", "T1562", "script", 90, 45, false),
    ]
}

const fn spec(
    id: &'static str,
    description: &'static str,
    agent_cible: &'static str,
    phase: PhaseKind,
    anomaly_ref: &'static str,
    mitre: &'static str,
    tool: &'static str,
    max_duration_secs: u64,
    detection_timeout_secs: u64,
    pipeline_default: bool,
) -> ScenarioSpec {
    ScenarioSpec {
        id,
        description,
        agent_cible,
        phase,
        anomaly_ref,
        mitre,
        tool,
        max_duration_secs,
        detection_timeout_secs,
        pipeline_default,
    }
}

pub fn all_scenarios() -> &'static [ScenarioSpec] {
    CATALOG.get_or_init(build_catalog)
}

pub fn find(id: &str) -> Option<&'static ScenarioSpec> {
    all_scenarios().iter().find(|s| s.id == id)
}

pub fn default_for_phase(phase: PhaseKind) -> Option<&'static ScenarioSpec> {
    all_scenarios()
        .iter()
        .find(|s| s.phase == phase && s.pipeline_default)
}

pub fn detection_timeout_for(scenario_id: &str, fallback: u64) -> u64 {
    find(scenario_id)
        .map(|s| s.detection_timeout_secs)
        .unwrap_or(fallback)
}

pub fn max_duration_for(scenario_id: &str) -> u64 {
    find(scenario_id).map(|s| s.max_duration_secs).unwrap_or(300)
}

/// Charge le JSON embarqué (validation doc / outils externes).
pub fn load_json_catalog() -> Option<serde_json::Value> {
    let raw = include_str!("../../knowledge/attacks_catalog.json");
    serde_json::from_str(raw).ok()
}

#[derive(Debug, Deserialize)]
struct JsonCatalog {
    scenarios: Vec<JsonScenario>,
}

#[derive(Debug, Deserialize)]
struct JsonScenario {
    id: String,
}

pub fn json_scenario_count() -> usize {
    let raw = include_str!("../../knowledge/attacks_catalog.json");
    serde_json::from_str::<JsonCatalog>(raw)
        .map(|c| c.scenarios.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_24_scenarios() {
        assert_eq!(all_scenarios().len(), 24);
        assert_eq!(json_scenario_count(), 24);
    }

    #[test]
    fn each_agent_represented() {
        assert!(all_scenarios().iter().any(|s| s.agent_cible == "filtreur"));
        assert!(all_scenarios().iter().any(|s| s.agent_cible == "analyseur"));
        assert!(all_scenarios().iter().any(|s| s.agent_cible == "chiffreur"));
        assert!(all_scenarios().iter().any(|s| s.agent_cible == "auditeur"));
    }
}
