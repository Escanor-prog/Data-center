#[derive(Debug)]
pub struct ScenarioInfo {
    pub id: &'static str,
    pub description: &'static str,
    pub agent_target: &'static str,
}

pub fn known_scenarios() -> &'static [ScenarioInfo] {
    &SCENARIOS
}

static SCENARIOS: [ScenarioInfo; 12] = [
    ScenarioInfo { id: "ddos_syn", description: "Inondation de paquets SYN", agent_target: "filtreur" },
    ScenarioInfo { id: "scan_ports", description: "Scan de ports externes", agent_target: "filtreur" },
    ScenarioInfo { id: "brute_force", description: "Force brute SSH inter-VM", agent_target: "analyseur" },
    ScenarioInfo { id: "exfiltration", description: "Fuite de données simulée", agent_target: "analyseur" },
    ScenarioInfo { id: "mitm", description: "Interception ARP poisoning", agent_target: "analyseur" },
    ScenarioInfo { id: "replay_attack", description: "Réutilisation de jeton expiré", agent_target: "analyseur" },
    ScenarioInfo { id: "saturation", description: "Remplissage disque sandbox", agent_target: "analyseur" },
    ScenarioInfo { id: "monitoring_fail", description: "Arrêt du monitoring sandbox", agent_target: "analyseur" },
    ScenarioInfo { id: "panne_reseau", description: "Coupure interface sandbox", agent_target: "analyseur" },
    ScenarioInfo { id: "cert_tls", description: "Connexion avec cert expiré", agent_target: "chiffreur" },
    ScenarioInfo { id: "mot_de_passe_faible", description: "Test mot de passe trop simple", agent_target: "chiffreur" },
    ScenarioInfo { id: "injection_base", description: "Injection de règle malformée", agent_target: "auditeur" },
];
