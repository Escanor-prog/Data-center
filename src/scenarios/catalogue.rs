use crate::scenarios::types::{AttacksCatalog, PhaseKind, ScenarioInfo};
use once_cell::sync::Lazy;

static CATALOG: Lazy<AttacksCatalog> = Lazy::new(|| {
    serde_json::from_str(include_str!("../../knowledge/attacks_catalog.json"))
        .expect("attacks_catalog.json invalide")
});

pub fn catalog() -> &'static AttacksCatalog {
    &CATALOG
}

pub fn known_scenarios() -> &'static [ScenarioInfo] {
    &CATALOG.scenarios
}

pub fn find_scenario(id: &str) -> Option<&'static ScenarioInfo> {
    known_scenarios().iter().find(|s| s.id == id)
}

pub fn default_for_phase(phase: PhaseKind) -> Option<&'static ScenarioInfo> {
    known_scenarios()
        .iter()
        .find(|s| s.phase == phase && s.pipeline_default)
        .or_else(|| known_scenarios().iter().find(|s| s.phase == phase))
}

pub fn max_duration_for(scenario_id: &str) -> u64 {
    find_scenario(scenario_id)
        .map(|s| s.max_duration_secs)
        .unwrap_or(300)
}

pub fn detection_timeout_for(scenario_id: &str) -> u64 {
    find_scenario(scenario_id)
        .map(|s| s.detection_timeout_secs)
        .unwrap_or(catalog().timeouts_policy.detection_default_secs)
}

pub fn detection_timeout_for_phase(phase: u8) -> u64 {
    let p = &catalog().timeouts_policy;
    match phase {
        1 => p.detection_phase1_secs,
        2 => p.detection_phase2_secs,
        3 => p.detection_phase3_secs,
        _ => p.detection_default_secs,
    }
}

pub fn global_pipeline_max_secs() -> u64 {
    catalog().timeouts_policy.global_pipeline_max_secs
}

pub fn scenarios_by_agent(agent: &str) -> Vec<&'static ScenarioInfo> {
    known_scenarios()
        .iter()
        .filter(|s| s.agent_cible == agent)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_25_scenarios() {
        assert_eq!(known_scenarios().len(), 25);
    }

    #[test]
    fn every_agent_has_scenarios() {
        for agent in ["filtreur", "analyseur", "chiffreur", "auditeur"] {
            assert!(
                !scenarios_by_agent(agent).is_empty(),
                "aucun scénario pour {agent}"
            );
        }
    }
}
