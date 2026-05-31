use axum::http::{HeaderMap, StatusCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallerRole {
    Auditeur,
    Decideur,
}

pub fn check_token(headers: &HeaderMap, expected: &str, role: CallerRole) -> Result<(), StatusCode> {
    let _ = role;
    match headers.get("x-agent-token") {
        Some(value) => {
            let token = value.to_str().unwrap_or_default();
            if token == expected {
                Ok(())
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => Err(StatusCode::FORBIDDEN),
    }
}

pub fn check_auditeur(headers: &HeaderMap, token: &str) -> Result<(), StatusCode> {
    check_token(headers, token, CallerRole::Auditeur)
}

pub fn check_decideur(headers: &HeaderMap, token: &str) -> Result<(), StatusCode> {
    check_token(headers, token, CallerRole::Decideur)
}
