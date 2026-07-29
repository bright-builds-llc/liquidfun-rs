use serde_json::{Value, json};

use super::*;
use crate::{HarnessLimits, RecordLimit, RequestId, ScenarioId, encode_jsonl};

const REQUEST: &[u8] =
    include_bytes!("../../../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn fixture_value() -> Value {
    serde_json::from_slice(REQUEST).expect("checked-in rigid-world request should be JSON")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should serialize");
    bytes.push(b'\n');
    bytes
}

fn timeline_mut<'a>(value: &'a mut Value, family: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == family)
        .expect("fixture should contain requested witness family")
}

fn action_mut<'a>(value: &'a mut Value, action_id: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["actions"]
                .as_array_mut()
                .expect("fixture actions should be an array")
        })
        .find(|action| action["action_id"] == action_id)
        .expect("fixture should contain requested action")
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "test call sites construct owned JSON action values inline"
)]
fn insert_non_colliding_action(value: &mut Value, action_id: &str, action: Value) {
    let actions = timeline_mut(value, "non_colliding_body_fixture_lifecycle")["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let insert_at = actions
        .iter()
        .position(|record| record["action"]["kind"] == "destroy_fixture")
        .expect("fixture should contain destruction actions");
    actions.insert(
        insert_at,
        json!({ "action_id": action_id, "phase": "phase7-contract", "action": action }),
    );
}

include!("tests/phase6.rs");
include!("tests/phase7.rs");
include!("tests/phase8.rs");
