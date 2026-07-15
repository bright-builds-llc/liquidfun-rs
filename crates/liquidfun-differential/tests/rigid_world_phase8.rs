//! Native Phase 8 rigid-world adapter integration tests.

use liquidfun_differential::{NativeRigidWorldExecutor, rigid_world_cpp_adapter_gate_reason};
use liquidfun_test_protocol::{
    HarnessLimits, RigidContactIdentity, RigidJointBranchState, RigidJointKind, RigidJointSnapshot,
    RigidLifecycleObservation, RigidLifecycleObservationKind, RigidWorldObservation,
    RigidWorldWitnessFamily, ScenarioId, decode_rigid_world_request_jsonl,
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

fn joint_snapshot_is_nontrivial(snapshot: &RigidJointSnapshot) -> bool {
    snapshot.branch_state != RigidJointBranchState::Inactive
        || snapshot.coordinate_bits.to_f32() != 0.0
        || snapshot.speed_bits.to_f32() != 0.0
        || snapshot.reaction_force.x_bits.to_f32() != 0.0
        || snapshot.reaction_force.y_bits.to_f32() != 0.0
        || snapshot.reaction_torque_bits.to_f32() != 0.0
}

#[test]
fn native_phase8_corpus_emits_nontrivial_live_records_for_every_joint_kind() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the complete Phase 8 request should execute natively");
    let mut snapshots = Vec::<&RigidJointSnapshot>::new();
    for snapshot in result
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .flat_map(|checkpoint| &checkpoint.observations)
        .filter_map(|observation| match observation {
            RigidWorldObservation::Joint { snapshot } => Some(snapshot),
            _ => None,
        })
    {
        if let Some(existing) = snapshots
            .iter_mut()
            .find(|existing| existing.joint_id == snapshot.joint_id)
        {
            *existing = snapshot;
        } else {
            snapshots.push(snapshot);
        }
    }

    // Assert
    for kind in RigidJointKind::ALL {
        assert!(
            snapshots
                .iter()
                .filter(|snapshot| snapshot.joint_kind == kind)
                .any(|snapshot| joint_snapshot_is_nontrivial(snapshot)),
            "{kind:?} must expose a nontrivial public post-step record"
        );
    }
}

#[test]
fn native_phase8_gear_records_pin_all_four_live_topologies() {
    // Arrange
    let request = request();
    let combinations = [
        (RigidJointKind::Revolute, RigidJointKind::Revolute),
        (RigidJointKind::Revolute, RigidJointKind::Prismatic),
        (RigidJointKind::Prismatic, RigidJointKind::Revolute),
        (RigidJointKind::Prismatic, RigidJointKind::Prismatic),
    ];

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the four-body gear corpus should execute natively");
    let observations = result
        .timelines()
        .iter()
        .find(|timeline| {
            timeline.witness_family == RigidWorldWitnessFamily::GearDependenciesAndFourBodySolver
        })
        .expect("gear result should exist")
        .checkpoints[0]
        .observations
        .iter()
        .filter_map(|observation| match observation {
            RigidWorldObservation::Joint { snapshot } => Some(snapshot),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Assert
    for (index, expected_kinds) in combinations.into_iter().enumerate() {
        let source_a_id = scenario_id(&format!("gear-{index}-source-a"));
        let source_b_id = scenario_id(&format!("gear-{index}-source-b"));
        let gear_id = scenario_id(&format!("gear-{index}-joint"));
        let source_a = observations
            .iter()
            .rev()
            .find(|snapshot| snapshot.joint_id == source_a_id)
            .expect("gear source A should be observed after the live step");
        let source_b = observations
            .iter()
            .rev()
            .find(|snapshot| snapshot.joint_id == source_b_id)
            .expect("gear source B should be observed after the live step");
        let gear = observations
            .iter()
            .rev()
            .find(|snapshot| snapshot.joint_id == gear_id)
            .expect("gear should be observed after the live step");

        assert_eq!((source_a.joint_kind, source_b.joint_kind), expected_kinds);
        assert_eq!(gear.dependencies.as_ref(), [source_a_id, source_b_id]);
        assert_eq!(
            (&gear.body_a_id, &gear.body_b_id),
            (
                &scenario_id(&format!("gear-{index}-moving-a")),
                &scenario_id(&format!("gear-{index}-moving-b")),
            )
        );
        assert!(joint_snapshot_is_nontrivial(gear));
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
fn native_phase8_solver_only_families_reject_incidental_contacts() {
    // Arrange
    let request = request();
    let solver_only = [
        RigidWorldWitnessFamily::JointDefinitionsAndMutations,
        RigidWorldWitnessFamily::RevolutePrismaticLimitsAndMotors,
        RigidWorldWitnessFamily::DistancePulleyMouseConstraints,
        RigidWorldWitnessFamily::WheelWeldFrictionRopeMotorConstraints,
        RigidWorldWitnessFamily::GearDependenciesAndFourBodySolver,
    ];

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the solver-only corpus should execute without incidental contacts");

    // Assert
    for family in solver_only {
        let timeline = result
            .timelines()
            .iter()
            .find(|timeline| timeline.witness_family == family)
            .expect("solver-only result should exist");
        assert!(
            timeline
                .checkpoints
                .iter()
                .all(|checkpoint| checkpoint.contacts.is_empty())
        );
        assert!(timeline.checkpoints.iter().all(|checkpoint| {
            checkpoint
                .observations
                .iter()
                .all(|observation| !matches!(observation, RigidWorldObservation::Lifecycle { .. }))
        }));
    }
}

#[test]
fn native_phase8_executes_every_closed_joint_mutation() {
    // Arrange
    let vector = json!({ "x_bits": 2.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() });
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
            json!({ "kind": "motor_speed", "speed_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "max_motor_force", "force_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "max_motor_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-distance",
            json!({ "kind": "length", "length_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-weld",
            json!({ "kind": "frequency", "frequency_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "damping_ratio", "damping_ratio_bits": 0.25_f32.to_bits() }),
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
            json!({ "kind": "max_torque", "torque_bits": 2.0_f32.to_bits() }),
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
            json!({ "kind": "correction_factor", "factor_bits": 0.75_f32.to_bits() }),
        ),
    ];

    // Act
    let results = mutations.map(|(joint_id, mutation)| {
        let mut value = request_value();
        if matches!(
            mutation["kind"].as_str(),
            Some("limit_enabled" | "motor_enabled")
        ) {
            let declaration = timeline_mut(&mut value, "joint_definitions_and_mutations")["joints"]
                .as_array_mut()
                .expect("fixture joints should be an array")
                .iter_mut()
                .find(|joint| joint["joint_id"] == joint_id)
                .expect("mutation target declaration should exist");
            let field = if mutation["kind"] == "limit_enabled" {
                "limit_enabled"
            } else {
                "motor_enabled"
            };
            declaration["definition"][field] = json!(false);
        }
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

fn lifecycle_for(
    result: &liquidfun_test_protocol::RigidWorldResultRecord,
    family: RigidWorldWitnessFamily,
) -> Vec<RigidLifecycleObservation> {
    result
        .timelines()
        .iter()
        .find(|timeline| timeline.witness_family == family)
        .expect("requested Phase 8 result should exist")
        .checkpoints
        .iter()
        .flat_map(|checkpoint| &checkpoint.observations)
        .filter_map(|observation| match observation {
            RigidWorldObservation::Lifecycle { event } => Some(event.clone()),
            _ => None,
        })
        .collect()
}

fn scenario_id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test identity should validate")
}

fn contact(fixture_a_id: &str, fixture_b_id: &str) -> RigidContactIdentity {
    RigidContactIdentity::new(
        scenario_id(fixture_a_id),
        0,
        scenario_id(fixture_b_id),
        0,
        1,
    )
    .expect("test contact identity should validate")
}

fn contact_event(
    ordinal: u32,
    kind: RigidLifecycleObservationKind,
    contact: &RigidContactIdentity,
) -> RigidLifecycleObservation {
    RigidLifecycleObservation {
        ordinal,
        kind,
        maybe_contact: Some(contact.clone()),
        maybe_entity_id: None,
    }
}

fn entity_event(
    ordinal: u32,
    kind: RigidLifecycleObservationKind,
    entity_id: &str,
) -> RigidLifecycleObservation {
    RigidLifecycleObservation {
        ordinal,
        kind,
        maybe_contact: None,
        maybe_entity_id: Some(scenario_id(entity_id)),
    }
}

#[test]
fn native_phase8_preserves_callback_lifecycle_order_and_multiplicity() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the callback corpus should execute through the native hook");
    let lifecycle = lifecycle_for(
        &result,
        RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming,
    );
    let callback_contact = contact("callback-fa", "callback-fb");

    // Assert
    assert_eq!(
        lifecycle,
        [
            entity_event(
                0,
                RigidLifecycleObservationKind::FilterDecision,
                "callback-fa"
            ),
            entity_event(
                1,
                RigidLifecycleObservationKind::FilterDecision,
                "callback-fa"
            ),
            contact_event(
                2,
                RigidLifecycleObservationKind::ContactCreated,
                &callback_contact
            ),
            contact_event(
                3,
                RigidLifecycleObservationKind::BeginContact,
                &callback_contact
            ),
            contact_event(
                4,
                RigidLifecycleObservationKind::PreSolve,
                &callback_contact
            ),
            contact_event(
                5,
                RigidLifecycleObservationKind::PostSolve,
                &callback_contact
            ),
            contact_event(
                6,
                RigidLifecycleObservationKind::PreSolve,
                &callback_contact
            ),
            contact_event(
                7,
                RigidLifecycleObservationKind::PostSolve,
                &callback_contact
            ),
            contact_event(
                8,
                RigidLifecycleObservationKind::PreSolve,
                &callback_contact
            ),
        ]
    );
}

#[test]
fn native_phase8_preserves_destruction_lifecycle_order_and_multiplicity() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the destruction corpus should execute through owned mutation reports");
    let lifecycle = lifecycle_for(
        &result,
        RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades,
    );
    let touching_contact = contact("destruction-moving-a-fixture", "destruction-base-b-fixture");

    // Assert
    assert_eq!(
        lifecycle,
        [
            entity_event(
                0,
                RigidLifecycleObservationKind::FilterDecision,
                "destruction-base-a-fixture",
            ),
            entity_event(
                1,
                RigidLifecycleObservationKind::FilterDecision,
                "destruction-moving-a-fixture",
            ),
            contact_event(
                2,
                RigidLifecycleObservationKind::ContactCreated,
                &touching_contact,
            ),
            contact_event(
                3,
                RigidLifecycleObservationKind::BeginContact,
                &touching_contact,
            ),
            contact_event(
                4,
                RigidLifecycleObservationKind::PreSolve,
                &touching_contact
            ),
            contact_event(
                5,
                RigidLifecycleObservationKind::PostSolve,
                &touching_contact
            ),
            contact_event(
                6,
                RigidLifecycleObservationKind::PreSolve,
                &touching_contact
            ),
            contact_event(
                7,
                RigidLifecycleObservationKind::PostSolve,
                &touching_contact
            ),
            entity_event(
                8,
                RigidLifecycleObservationKind::JointGoodbye,
                "destruction-dependent-gear",
            ),
            contact_event(
                9,
                RigidLifecycleObservationKind::EndContact,
                &touching_contact
            ),
            contact_event(
                10,
                RigidLifecycleObservationKind::ContactDestroyed,
                &touching_contact,
            ),
            entity_event(
                11,
                RigidLifecycleObservationKind::FixtureGoodbye,
                "destruction-moving-b-fixture",
            ),
            entity_event(
                12,
                RigidLifecycleObservationKind::BodyDestroyed,
                "destruction-moving-b",
            ),
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
