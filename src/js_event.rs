use cqrs_es::persist::SerializedEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
pub struct JsEvent {
    pub aggregate_id: String,
    pub sequence: usize,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_version: String,
    pub payload: Value,
    pub metadata: Value,
}

impl Into<SerializedEvent> for JsEvent {
    fn into(self) -> SerializedEvent {
        SerializedEvent {
            aggregate_id: self.aggregate_id,
            sequence: self.sequence,
            aggregate_type: self.aggregate_type,
            event_type: self.event_type,
            event_version: self.event_version,
            payload: self.payload,
            metadata: self.metadata,
        }
    }
}

impl From<SerializedEvent> for JsEvent {
    fn from(value: SerializedEvent) -> JsEvent {
        JsEvent {
            aggregate_id: value.aggregate_id,
            sequence: value.sequence,
            aggregate_type: value.aggregate_type,
            event_type: value.event_type,
            event_version: value.event_version,
            payload: value.payload,
            metadata: value.metadata,
        }
    }
}
