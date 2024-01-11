use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize)]
pub struct Event {
    pub name: String,
    pub data: Value,
}

#[derive(Clone, Deserialize)]
pub struct Payload {
    pub event: String,
    pub payload: Value,
}
