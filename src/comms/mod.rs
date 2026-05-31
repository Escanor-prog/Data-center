pub mod analyseur;
pub mod auditeur;
pub mod chiffreur;
pub mod decideur;
pub mod dto;
pub mod health;

use crate::config::Config;

pub struct AgentClients {
    pub auditeur: auditeur::AuditeurClient,
    pub decideur: decideur::DecideurClient,
    pub analyseur: analyseur::AnalyseurClient,
    pub chiffreur: chiffreur::ChiffreurClient,
}

impl AgentClients {
    pub fn from_config(cfg: &Config) -> Self {
        let dry = cfg.dry_run;
        Self {
            auditeur: auditeur::AuditeurClient::new(
                cfg.auditeur_url.clone(),
                cfg.agent_token.clone(),
                cfg.backup_events_path.clone(),
                dry,
            ),
            decideur: decideur::DecideurClient::new(
                cfg.decideur_url.clone(),
                cfg.decideur_token().to_string(),
                dry,
            ),
            analyseur: analyseur::AnalyseurClient::new(
                cfg.analyseur_url.clone(),
                cfg.agent_token.clone(),
                dry,
            ),
            chiffreur: chiffreur::ChiffreurClient::new(
                cfg.chiffreur_url.clone(),
                cfg.agent_token.clone(),
                dry,
            ),
        }
    }
}
