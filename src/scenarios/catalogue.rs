/// Structure décrivant un scénario de simulation de sécurité.
///
/// Pour des raisons de compatibilité et de transfert de propriété (ownership) avec les autres modules
/// du projet (notamment `api/routes.rs` et `control/constraints.rs`), les champs utilisent des types `&'static str`
/// plutôt que des `String` alloués dynamiquement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioInfo {
    pub id: &'static str,
    pub description: &'static str,
    pub agent_cible: &'static str,
}

/// Retourne la liste complète de tous les scénarios connus du simulateur.
pub fn known_scenarios() -> &'static [ScenarioInfo] {
    &SCENARIOS
}

/// Catalogue statique contenant les 12 scénarios requis.
static SCENARIOS: [ScenarioInfo; 12] = [
    ScenarioInfo {
        id: "ddos_syn",
        description: "Inondation de paquets SYN",
        agent_cible: "filtreur",
    },
    ScenarioInfo {
        id: "scan_ports",
        description: "Scan de ports externes",
        agent_cible: "filtreur",
    },
    ScenarioInfo {
        id: "brute_force",
        description: "Force brute SSH inter-VM",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "exfiltration",
        description: "Fuite de données simulée",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "mitm",
        description: "Interception ARP poisoning",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "replay_attack",
        description: "Réutilisation de jeton expiré",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "saturation",
        description: "Remplissage disque sandbox",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "monitoring_fail",
        description: "Arrêt du monitoring sandbox",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "panne_reseau",
        description: "Coupure interface sandbox",
        agent_cible: "analyseur",
    },
    ScenarioInfo {
        id: "cert_tls",
        description: "Connexion avec certificat expiré",
        agent_cible: "chiffreur",
    },
    ScenarioInfo {
        id: "mot_de_passe_faible",
        description: "Test mot de passe faible",
        agent_cible: "chiffreur",
    },
    ScenarioInfo {
        id: "injection_base",
        description: "Injection règle malformée",
        agent_cible: "auditeur",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_scenarios_count() {
        let scenarios = known_scenarios();
        assert_eq!(scenarios.len(), 12);
    }

    #[test]
    fn test_scenario_fields() {
        let scenarios = known_scenarios();
        assert_eq!(scenarios[0].id, "ddos_syn");
        assert_eq!(scenarios[0].agent_cible, "filtreur");
    }
}
