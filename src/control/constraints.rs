use crate::scenarios::catalogue;
use std::collections::HashSet;

pub fn check_no_active_simulation(active: bool) -> Result<(), String> {
    if active {
        Err("Une simulation est déjà en cours".to_string())
    } else {
        Ok(())
    }
}

pub fn check_token(received: &str, expected: &str) -> Result<(), String> {
    if received == expected {
        Ok(())
    } else {
        Err("Token invalide".to_string())
    }
}

pub fn check_scenario_known(scenario: &str) -> Result<(), String> {
    let known: HashSet<&str> = catalogue::known_scenarios()
        .iter()
        .map(|s| s.id)
        .collect();

    if known.contains(scenario) {
        Ok(())
    } else {
        Err(format!("Scénario inconnu : {}", scenario))
    }
}

pub fn max_duration_for(_scenario: &str) -> u64 {
    300
}
