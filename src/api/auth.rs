use axum::http::HeaderMap;
use axum::http::StatusCode;

pub fn check_token(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
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
