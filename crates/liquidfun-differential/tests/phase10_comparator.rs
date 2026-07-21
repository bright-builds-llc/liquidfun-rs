//! Closed Phase 10 comparator policy, mutation, and diagnostic witnesses.

use liquidfun_differential::{
    PHASE10_POLICY_REGISTRY, PHASE10_REQUIRED_POLICY_PATHS, Phase10ComparatorError,
    Phase10ComparisonMode, Phase10ComparisonOutcome, compare_phase10_observations,
    validate_phase10_policy_registry,
};
use liquidfun_test_protocol::{
    FloatBits, Phase10BehaviorLeaf, Phase10BodyContact, Phase10Event, Phase10EventKind,
    Phase10GroupSnapshot, Phase10Observation, Phase10PairSnapshot, Phase10ParticleContact,
    Phase10ParticleSnapshot, Phase10Provenance, Phase10SemanticOutcome, Phase10StateObservation,
    Phase10TriadSnapshot, Phase10ValidationKind, Phase10Witness, Phase10WitnessObservation,
    ScenarioId, TransformBits, Vec2Bits, WitnessRole,
};

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test identifier should be valid")
}

fn bits(value: f32) -> FloatBits {
    FloatBits::from_f32(value)
}

fn vec2(x: f32, y: f32) -> Vec2Bits {
    Vec2Bits {
        x_bits: bits(x),
        y_bits: bits(y),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one complete valid observation fixture exercises every result field family"
)]
fn observation() -> Phase10Observation {
    let particles = [
        Phase10ParticleSnapshot {
            particle_id: id("particle-1"),
            system_id: id("system-1"),
            group_id: id("group-1"),
            position: vec2(0.0, 0.0),
            velocity: vec2(1.0, 0.0),
            flags_bits: 0x8,
            color: [10, 20, 30, 40],
            weight_bits: bits(0.5),
        },
        Phase10ParticleSnapshot {
            particle_id: id("particle-2"),
            system_id: id("system-1"),
            group_id: id("group-1"),
            position: vec2(1.0, 0.0),
            velocity: vec2(0.0, 1.0),
            flags_bits: 0x8,
            color: [40, 30, 20, 10],
            weight_bits: bits(0.75),
        },
        Phase10ParticleSnapshot {
            particle_id: id("particle-3"),
            system_id: id("system-1"),
            group_id: id("group-1"),
            position: vec2(0.0, 1.0),
            velocity: vec2(-1.0, 0.0),
            flags_bits: 0x8,
            color: [1, 2, 3, 4],
            weight_bits: bits(1.0),
        },
    ];
    Phase10Observation::State {
        state: Phase10StateObservation {
            provenance: Phase10Provenance {
                extension_version: 1,
                generator_id: id("phase10-comparator"),
                generator_version: id("v1"),
                upstream_revision: id("upstream-revision"),
                toolchain_id: id("toolchain"),
                seed: 42,
            },
            outcome: Phase10SemanticOutcome::Completed,
            groups: vec![Phase10GroupSnapshot {
                ordinal: 0,
                group_id: id("group-1"),
                system_id: id("system-1"),
                member_ids: particles
                    .iter()
                    .map(|particle| particle.particle_id.clone())
                    .collect(),
                group_flags_bits: 0x1,
                transform: TransformBits {
                    position: vec2(0.25, 0.5),
                    angle_bits: bits(0.125),
                },
                center: vec2(0.5, 0.5),
                linear_velocity: vec2(0.0, 0.25),
                angular_velocity_bits: bits(0.5),
                mass_bits: bits(3.0),
                inertia_bits: bits(2.0),
                maybe_depths_bits: Some(vec![bits(0.0), bits(0.5), bits(1.0)].into_boxed_slice()),
            }]
            .into_boxed_slice(),
            particles: particles.into(),
            pairs: vec![Phase10PairSnapshot {
                ordinal: 0,
                particle_a_id: id("particle-1"),
                particle_b_id: id("particle-2"),
                flags_bits: 0x8,
                strength_bits: bits(1.0),
                distance_bits: bits(1.0),
            }]
            .into_boxed_slice(),
            triads: vec![Phase10TriadSnapshot {
                ordinal: 0,
                particle_a_id: id("particle-1"),
                particle_b_id: id("particle-2"),
                particle_c_id: id("particle-3"),
                flags_bits: 0x10,
                strength_bits: bits(0.75),
                pa: vec2(-0.5, -0.5),
                pb: vec2(0.5, -0.5),
                pc: vec2(0.0, 0.5),
                ka_bits: bits(1.0),
                kb_bits: bits(2.0),
                kc_bits: bits(3.0),
                s_bits: bits(0.5),
            }]
            .into_boxed_slice(),
            particle_contacts: vec![Phase10ParticleContact {
                ordinal: 0,
                system_id: id("system-1"),
                particle_a_id: id("particle-1"),
                particle_b_id: id("particle-2"),
                flags_bits: 0x8,
                weight_bits: bits(0.5),
                normal: vec2(1.0, 0.0),
            }]
            .into_boxed_slice(),
            body_contacts: vec![Phase10BodyContact {
                ordinal: 0,
                system_id: id("system-1"),
                particle_id: id("particle-1"),
                body_id: id("body-1"),
                fixture_id: id("fixture-1"),
                weight_bits: bits(0.5),
                normal: vec2(0.0, 1.0),
                mass_bits: bits(2.0),
            }]
            .into_boxed_slice(),
            events: vec![Phase10Event {
                ordinal: 0,
                kind: Phase10EventKind::GroupCreated,
                system_id: id("system-1"),
                maybe_group_id: Some(id("group-1")),
                maybe_particle_id: None,
                maybe_other_particle_id: None,
                maybe_body_id: None,
            }]
            .into_boxed_slice(),
            witnesses: witnesses(),
        },
    }
}

fn witnesses() -> Box<[Phase10Witness]> {
    vec![
        witness(
            0,
            Phase10BehaviorLeaf::Water,
            WitnessRole::Control,
            Phase10WitnessObservation::ControlUnchanged,
        ),
        witness(
            1,
            Phase10BehaviorLeaf::Spring,
            WitnessRole::Activation,
            Phase10WitnessObservation::FlagActivated { flags_bits: 0x8 },
        ),
        witness(
            2,
            Phase10BehaviorLeaf::Elastic,
            WitnessRole::Interaction,
            Phase10WitnessObservation::ParticleVelocity {
                particle_id: id("particle-1"),
                before: vec2(0.0, 0.0),
                after: vec2(1.0, 0.0),
            },
        ),
        witness(
            3,
            Phase10BehaviorLeaf::Viscous,
            WitnessRole::Interaction,
            Phase10WitnessObservation::Scalar {
                value_bits: bits(0.25),
            },
        ),
        witness(
            4,
            Phase10BehaviorLeaf::GroupCreate,
            WitnessRole::Activation,
            Phase10WitnessObservation::Count { value: 3 },
        ),
        witness(
            5,
            Phase10BehaviorLeaf::GroupDestroy,
            WitnessRole::Activation,
            Phase10WitnessObservation::Occurrence { event_ordinal: 0 },
        ),
        witness(
            6,
            Phase10BehaviorLeaf::Reactive,
            WitnessRole::Interaction,
            Phase10WitnessObservation::Topology {
                pair_count: 1,
                triad_count: 1,
            },
        ),
    ]
    .into_boxed_slice()
}

fn witness(
    ordinal: u32,
    behavior_leaf: Phase10BehaviorLeaf,
    role: WitnessRole,
    observation: Phase10WitnessObservation,
) -> Phase10Witness {
    Phase10Witness {
        ordinal,
        behavior_leaf,
        role,
        observation,
    }
}

fn state(observation: &mut Phase10Observation) -> &mut Phase10StateObservation {
    let Phase10Observation::State { state } = observation;
    state
}

fn mismatch_path(expected: &Phase10Observation, actual: &Phase10Observation) -> &'static str {
    let result = compare_phase10_observations(Phase10ComparisonMode::D1Semantic, expected, actual)
        .expect("well-formed observations should compare");
    let Phase10ComparisonOutcome::PhysicsMismatch(mismatch) = result else {
        panic!("mutation should produce a semantic mismatch");
    };
    assert_eq!(mismatch.scenario(), "phase10-comparator");
    assert!(!mismatch.entity().is_empty());
    assert!(!mismatch.expected().is_empty());
    assert!(!mismatch.actual().is_empty());
    mismatch.semantic_path()
}

#[test]
fn policy_registry_is_complete_ordered_and_rejects_every_open_binding_class() {
    // Arrange
    let complete = PHASE10_REQUIRED_POLICY_PATHS;
    let missing = &complete[..complete.len() - 1];
    let mut duplicate = complete.to_vec();
    duplicate.push(complete[0]);
    let forbidden = ["phase10.pass_id"];
    let unknown = ["phase10.unknown"];
    let wildcard = ["phase10.*"];

    // Act
    let valid = validate_phase10_policy_registry(complete);
    let invalid = [
        validate_phase10_policy_registry(missing),
        validate_phase10_policy_registry(&duplicate),
        validate_phase10_policy_registry(&forbidden),
        validate_phase10_policy_registry(&unknown),
        validate_phase10_policy_registry(&wildcard),
    ];

    // Assert
    assert_eq!(PHASE10_POLICY_REGISTRY.len(), complete.len());
    assert_eq!(
        valid.expect("closed registry should pass").as_ref(),
        complete
    );
    assert!(
        invalid
            .into_iter()
            .all(|result| matches!(result, Err(Phase10ComparatorError::PolicyRegistry { .. })))
    );
}

#[test]
fn d0_requires_canonical_byte_identity_even_inside_d1_tolerance() {
    // Arrange
    let expected = observation();
    let mut actual = expected.clone();
    let current = state(&mut actual).particles[0].position.x_bits.bits();
    state(&mut actual).particles[0].position.x_bits = FloatBits::new(current + 1);

    // Act
    let d0 =
        compare_phase10_observations(Phase10ComparisonMode::D0ByteIdentity, &expected, &actual)
            .expect("valid D0 values should compare");
    let d1 = compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
        .expect("valid D1 values should compare");

    // Assert
    assert!(matches!(d0, Phase10ComparisonOutcome::PhysicsMismatch(_)));
    assert!(matches!(d1, Phase10ComparisonOutcome::Match { .. }));
}

#[test]
fn numeric_policies_accept_boundary_and_reject_one_over() {
    // Arrange
    let mut expected = observation();
    state(&mut expected).particles[0].weight_bits = bits(0.0);
    let mut ulp_boundary = expected.clone();
    let position = state(&mut ulp_boundary).particles[0].position.x_bits.bits();
    state(&mut ulp_boundary).particles[0].position.x_bits = FloatBits::new(position + 4);
    let mut ulp_over = expected.clone();
    state(&mut ulp_over).particles[0].position.x_bits = FloatBits::new(position + 5);
    let mut absolute_boundary = expected.clone();
    state(&mut absolute_boundary).particles[0].weight_bits = bits(1.0e-6);
    let boundary_bits = bits(1.0e-6).bits();
    let mut absolute_over = expected.clone();
    state(&mut absolute_over).particles[0].weight_bits = FloatBits::new(boundary_bits + 1);

    // Act
    let accepted = [ulp_boundary, absolute_boundary].map(|actual| {
        compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
    });
    let rejected = [ulp_over, absolute_over].map(|actual| mismatch_path(&expected, &actual));

    // Assert
    assert!(
        accepted
            .into_iter()
            .all(|result| matches!(result, Ok(Phase10ComparisonOutcome::Match { .. })))
    );
    assert_eq!(
        rejected,
        ["phase10.particle.position", "phase10.particle.weight"]
    );
}

#[test]
fn comparator_reports_first_contextual_mismatch_across_every_record_family() {
    // Arrange
    let expected = observation();
    let mutations: Vec<(Phase10Observation, &'static str)> = vec![
        (
            {
                let mut v = expected.clone();
                state(&mut v).provenance.seed += 1;
                v
            },
            "phase10.provenance",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).outcome = Phase10SemanticOutcome::Rejected {
                    reason: liquidfun_test_protocol::Phase10RejectionReason::Locked,
                };
                v
            },
            "phase10.outcome",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).groups[0].group_flags_bits = 0x2;
                v
            },
            "phase10.group.flags",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).particles[0].color[0] += 1;
                v
            },
            "phase10.particle.color",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).pairs[0].strength_bits = bits(0.5);
                v
            },
            "phase10.pair.strength",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).triads[0].ka_bits = bits(2.0);
                v
            },
            "phase10.triad.coefficient",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).particle_contacts[0].weight_bits = bits(0.75);
                v
            },
            "phase10.contact.weight",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).body_contacts[0].mass_bits = bits(3.0);
                v
            },
            "phase10.body_contact.mass",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).events[0].kind = Phase10EventKind::GroupDestroyed;
                v
            },
            "phase10.event.kind",
        ),
        (
            {
                let mut v = expected.clone();
                state(&mut v).witnesses[0].behavior_leaf = Phase10BehaviorLeaf::Wall;
                v
            },
            "phase10.witness.leaf",
        ),
    ];

    // Act
    let paths = mutations
        .iter()
        .map(|(actual, _)| mismatch_path(&expected, actual))
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        paths,
        mutations.iter().map(|(_, path)| *path).collect::<Vec<_>>()
    );
}

#[test]
fn malformed_nonfinite_duplicate_reordered_and_dropped_records_fail_closed() {
    // Arrange
    let expected = observation();
    let mut nonfinite = expected.clone();
    state(&mut nonfinite).groups[0].mass_bits = bits(f32::NAN);
    let mut duplicate = expected.clone();
    state(&mut duplicate).particles[1].particle_id = id("particle-1");
    let mut reordered = expected.clone();
    state(&mut reordered).particles.swap(0, 1);
    let mut dropped = expected.clone();
    state(&mut dropped).pairs = Box::new([]);
    let Phase10WitnessObservation::Topology { pair_count, .. } =
        &mut state(&mut dropped).witnesses[6].observation
    else {
        panic!("fixture witness should describe topology");
    };
    *pair_count = 0;

    // Act
    let invalid = [nonfinite, duplicate, reordered].map(|actual| {
        compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
    });
    let dropped_path = mismatch_path(&expected, &dropped);

    // Assert
    assert!(
        invalid
            .into_iter()
            .all(|result| matches!(result, Err(Phase10ComparatorError::ResultValidation { .. })))
    );
    assert_eq!(dropped_path, "phase10.pair.identity");
}

#[test]
fn diagnostic_identifies_operation_entity_field_index_and_stable_signature() {
    // Arrange
    let expected = observation();
    let mut actual = expected.clone();
    state(&mut actual).particles[1].velocity.y_bits = bits(4.0);

    // Act
    let first = compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
        .expect("valid observations should compare");
    let second =
        compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
            .expect("valid observations should compare");
    let (
        Phase10ComparisonOutcome::PhysicsMismatch(first),
        Phase10ComparisonOutcome::PhysicsMismatch(second),
    ) = (first, second)
    else {
        panic!("mutation should mismatch");
    };

    // Assert
    assert_eq!(first.semantic_path(), "phase10.particle.velocity");
    assert_eq!(first.operation(), "state");
    assert_eq!(first.entity(), "particle:particle-2");
    assert_eq!(first.index(), 3);
    assert_eq!(first.signature_sha256(), second.signature_sha256());
}

#[test]
fn nonfinite_validation_class_is_preserved() {
    // Arrange
    let expected = observation();
    let mut actual = expected.clone();
    state(&mut actual).particles[0].position.x_bits = bits(f32::INFINITY);

    // Act
    let error = compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &expected, &actual)
        .expect_err("non-finite observation must fail closed");

    // Assert
    assert_eq!(
        error,
        Phase10ComparatorError::ResultValidation {
            side: "actual",
            kind: Phase10ValidationKind::InvalidFloat,
        }
    );
}
