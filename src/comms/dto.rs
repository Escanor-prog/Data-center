use chrono::Utc;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_suffix() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let n = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    nanos ^ n.wrapping_mul(0x9E37_79B9)
}

pub fn new_request_id() -> String {
    format!("req-{:016x}", unique_suffix())
}

pub fn new_simulation_id() -> String {
    format!("SIM-{}", Utc::now().format("%Y%m%d-%H%M%S"))
}

pub fn new_simulation_token() -> String {
    format!("SIM-TOKEN-{:016x}", unique_suffix())
}

/// Enveloppe JSON commune (Document_APIs.txt).
pub fn envelope(message_type: &str, fields: Value) -> Value {
    let request_id = new_request_id();
    match fields {
        Value::Object(mut map) => {
            map.insert("request_id".into(), json!(request_id));
            map.insert("message_type".into(), json!(message_type));
            Value::Object(map)
        }
        other => json!({
            "request_id": request_id,
            "message_type": message_type,
            "payload": other
        }),
    }
}

pub fn log_event_body(simulation_id: &str, event_type: &str, data: Value) -> Value {
    envelope(
        "log_event",
        json!({
            "source_agent": "simulateur",
            "event_type": event_type,
            "timestamp": Utc::now().to_rfc3339(),
            "data": {
                "simulation_id": simulation_id,
                "details": data
            }
        }),
    )
}
