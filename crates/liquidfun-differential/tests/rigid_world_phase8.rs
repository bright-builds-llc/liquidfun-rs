//! Native Phase 8 rigid-world adapter integration tests.

use liquidfun_differential::{NativeRigidWorldExecutor, rigid_world_cpp_adapter_gate_reason};
use liquidfun_test_protocol::{
    HarnessLimits, RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS,
    RIGID_WORLD_VELOCITY_ITERATIONS, RigidJointKind, RigidLifecycleObservationKind,
    RigidWorldObservation, RigidWorldWitnessFamily, decode_rigid_world_request_jsonl,
};
use serde_json::{Value, json};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    decode_rigid_world_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("checked-in Phase 8 request should decode")
}

fn request_value() -> Value {
    serde_json::from_slice(REQUEST).expect("checked-in Phase 8 request should be JSON")
}

fn decode_value(value: &Value) -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut bytes = serde_json::to_vec(value).expect("modified Phase 8 request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("modified Phase 8 request should decode")
}

fn timeline_mut<'a>(value: &'a mut Value, family: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == family)
        .expect("requested timeline should exist")
}

#[test]
fn native_phase8_corpus_executes_deterministically() {
    // Arrange
    let request = request();

    // Act
    let first = NativeRigidWorldExecutor::execute(&request)
        .expect("the complete Phase 8 request should execute natively");
    let second = NativeRigidWorldExecutor::execute(&request)
        .expect("a fresh Phase 8 execution should reset all native state");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.timelines().len(), RigidWorldWitnessFamily::ALL.len());
}

#[test]
fn native_phase8_corpus_emits_every_joint_kind() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the complete Phase 8 request should execute natively");
    let observed = result
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .flat_map(|checkpoint| &checkpoint.observations)
        .filter_map(|observation| match observation {
            RigidWorldObservation::Joint { snapshot } => Some(snapshot.joint_kind),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Assert
    for kind in RigidJointKind::ALL {
        assert!(observed.contains(&kind), "missing {kind:?} observation");
    }
}

#[test]
fn native_phase8_corpus_emits_rope_reconstruction_and_diagnostics() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the complete Phase 8 request should execute natively");
    let observations = result
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .flat_map(|checkpoint| &checkpoint.observations)
        .collect::<Vec<_>>();

    // Assert
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Rope { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Reconstruction { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Diagnostics { .. }))
    );
}

#[test]
fn native_phase8_executes_every_closed_joint_mutation() {
    // Arrange
    let vector = json!({ "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() });
    let mutations = [
        (
            "joint-def-revolute",
            json!({ "kind": "limit_enabled", "enabled": true }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "limits", "lower_bits": 0.0_f32.to_bits(), "upper_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "motor_enabled", "enabled": true }),
        ),
        (
            "joint-def-revolute",
            json!({ "kind": "motor_speed", "speed_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "max_motor_force", "force_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "max_motor_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-distance",
            json!({ "kind": "length", "length_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-weld",
            json!({ "kind": "frequency", "frequency_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "damping_ratio", "damping_ratio_bits": 0.5_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "mouse_target", "target": vector }),
        ),
        (
            "joint-def-friction",
            json!({ "kind": "max_force", "force_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "max_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-gear",
            json!({ "kind": "gear_ratio", "ratio_bits": (-1.0_f32).to_bits() }),
        ),
        (
            "joint-def-rope-joint",
            json!({ "kind": "rope_max_length", "max_length_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "linear_offset", "offset": vector }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "angular_offset", "offset_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "correction_factor", "factor_bits": 0.5_f32.to_bits() }),
        ),
    ];

    // Act
    let results = mutations.map(|(joint_id, mutation)| {
        let mut value = request_value();
        let action = timeline_mut(&mut value, "joint_definitions_and_mutations")["actions"]
            .as_array_mut()
            .expect("timeline actions should be an array")
            .iter_mut()
            .find(|action| action["action_id"] == "joint-def-mutate")
            .expect("mutation action should exist");
        action["action"]["joint_id"] = json!(joint_id);
        action["action"]["mutation"] = mutation;
        NativeRigidWorldExecutor::execute(&decode_value(&value))
    });

    // Assert
    assert!(results.iter().all(Result::is_ok));
}

#[test]
fn native_phase8_applies_filter_and_pre_solve_directives_at_step_time() {
    // Arrange
    let mut value = request_value();
    let timeline = timeline_mut(&mut value, "contact_filter_listener_and_pre_solve_timing");
    timeline["actions"][5]["action"]["directive"]["enabled"] = json!(false);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array");
    actions.insert(
        6,
        json!({
            "action_id": "callback-step",
            "phase": "callback-step",
            "action": {
                "kind": "step",
                "timestep_bits": RIGID_WORLD_TIMESTEP_BITS,
                "velocity_iterations": RIGID_WORLD_VELOCITY_ITERATIONS,
                "position_iterations": RIGID_WORLD_POSITION_ITERATIONS
            }
        }),
    );
    timeline["checkpoints"][0]["after_action_id"] = json!("callback-step");
    timeline["checkpoints"][0]["phase"] = json!("callback-step");
    timeline["checkpoints"][0]["counts"]["contacts"] = json!(1);
    timeline["checkpoints"][0]["counts"]["manifold_points"] = json!(1);
    timeline["checkpoints"][0]["counts"]["events"] = json!(3);
    let request = decode_value(&value);

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("callback directives should execute through the native hook");
    let lifecycle = result
        .timelines()
        .iter()
        .find(|result| {
            result.witness_family == RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
        })
        .expect("callback result should exist")
        .checkpoints[0]
        .observations
        .iter()
        .filter_map(|observation| match observation {
            RigidWorldObservation::Lifecycle { event } => Some(event.kind),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        lifecycle,
        [
            RigidLifecycleObservationKind::FilterDecision,
            RigidLifecycleObservationKind::ContactCreated,
            RigidLifecycleObservationKind::BeginContact,
            RigidLifecycleObservationKind::PreSolve,
        ]
    );
}

#[test]
fn phase8_cpp_execution_gate_is_open_after_plan_08_13() {
    // Arrange
    let request = request();

    // Act
    let maybe_reason = rigid_world_cpp_adapter_gate_reason(&request);

    // Assert
    assert_eq!(maybe_reason, None);
}
