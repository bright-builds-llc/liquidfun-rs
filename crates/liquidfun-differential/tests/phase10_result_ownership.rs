//! Focused Phase 10 result ownership and topology validation regressions.

use liquidfun_test_protocol::{
    FloatBits, Phase10GroupSnapshot, Phase10Observation, Phase10PairSnapshot,
    Phase10ParticleSnapshot, Phase10Provenance, Phase10SemanticOutcome, Phase10StateObservation,
    Phase10TriadSnapshot, Phase10ValidationKind, ScenarioId, TransformBits, Vec2Bits,
};

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test semantic ID should be valid")
}

fn bits(value: f32) -> FloatBits {
    FloatBits::from_f32(value)
}

fn vector(x: f32, y: f32) -> Vec2Bits {
    Vec2Bits {
        x_bits: bits(x),
        y_bits: bits(y),
    }
}

#[test]
fn result_rejects_particle_group_id_that_disagrees_with_member_ownership() {
    // Arrange
    let mut observation = ownership_observation();
    let Phase10Observation::State { state } = &mut observation;
    state.particles[0].group_id = id("group-b");

    // Act
    let result = observation.validate_semantics();

    // Assert
    assert_eq!(
        result.map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidOwnership)
    );
}

#[test]
fn result_rejects_pair_that_spans_particle_systems() {
    // Arrange
    let mut observation = cross_system_ownership_observation();
    let Phase10Observation::State { state } = &mut observation;
    state.pairs = vec![Phase10PairSnapshot {
        ordinal: 0,
        particle_a_id: id("particle-a"),
        particle_b_id: id("particle-b"),
        flags_bits: 0,
        strength_bits: bits(1.0),
        distance_bits: bits(1.0),
    }]
    .into_boxed_slice();

    // Act
    let result = observation.validate_semantics();

    // Assert
    assert_eq!(
        result.map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidOwnership)
    );
}

#[test]
fn result_rejects_triad_that_spans_particle_systems() {
    // Arrange
    let mut observation = cross_system_ownership_observation();
    let Phase10Observation::State { state } = &mut observation;
    state.triads = vec![Phase10TriadSnapshot {
        ordinal: 0,
        particle_a_id: id("particle-a"),
        particle_b_id: id("particle-b"),
        particle_c_id: id("particle-c"),
        flags_bits: 0,
        strength_bits: bits(1.0),
        pa: vector(0.0, 0.0),
        pb: vector(1.0, 0.0),
        pc: vector(1.0, 1.0),
        ka_bits: bits(0.0),
        kb_bits: bits(0.0),
        kc_bits: bits(0.0),
        s_bits: bits(1.0),
    }]
    .into_boxed_slice();

    // Act
    let result = observation.validate_semantics();

    // Assert
    assert_eq!(
        result.map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidOwnership)
    );
}

fn ownership_observation() -> Phase10Observation {
    Phase10Observation::State {
        state: Phase10StateObservation {
            provenance: Phase10Provenance {
                extension_version: 1,
                generator_id: id("phase10-test-generator"),
                generator_version: id("v1"),
                upstream_revision: id("upstream-revision"),
                toolchain_id: id("rust-test-toolchain"),
                seed: 42,
            },
            outcome: Phase10SemanticOutcome::Completed,
            groups: vec![
                group_snapshot(0, "group-a", "system-a", &["particle-a"]),
                group_snapshot(1, "group-b", "system-a", &["particle-b", "particle-c"]),
            ]
            .into_boxed_slice(),
            particles: vec![
                particle_snapshot("particle-a", "group-a", "system-a"),
                particle_snapshot("particle-b", "group-b", "system-a"),
                particle_snapshot("particle-c", "group-b", "system-a"),
            ]
            .into_boxed_slice(),
            pairs: Box::new([]),
            triads: Box::new([]),
            particle_contacts: Box::new([]),
            body_contacts: Box::new([]),
            events: Box::new([]),
            witnesses: Box::new([]),
        },
    }
}

fn cross_system_ownership_observation() -> Phase10Observation {
    let mut observation = ownership_observation();
    let Phase10Observation::State { state } = &mut observation;
    state.groups[1].system_id = id("system-b");
    for particle in &mut state.particles[1..] {
        particle.system_id = id("system-b");
    }
    observation
}

fn group_snapshot(
    ordinal: u32,
    group_id: &str,
    system_id: &str,
    member_ids: &[&str],
) -> Phase10GroupSnapshot {
    Phase10GroupSnapshot {
        ordinal,
        group_id: id(group_id),
        system_id: id(system_id),
        member_ids: member_ids
            .iter()
            .map(|member_id| id(member_id))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        group_flags_bits: 0,
        transform: TransformBits {
            position: vector(0.0, 0.0),
            angle_bits: bits(0.0),
        },
        center: vector(0.0, 0.0),
        linear_velocity: vector(0.0, 0.0),
        angular_velocity_bits: bits(0.0),
        mass_bits: bits(0.0),
        inertia_bits: bits(0.0),
        maybe_depths_bits: None,
    }
}

fn particle_snapshot(
    particle_id: &str,
    group_id: &str,
    system_id: &str,
) -> Phase10ParticleSnapshot {
    Phase10ParticleSnapshot {
        particle_id: id(particle_id),
        system_id: id(system_id),
        group_id: id(group_id),
        position: vector(0.0, 0.0),
        velocity: vector(0.0, 0.0),
        flags_bits: 0,
        color: [0; 4],
        weight_bits: bits(0.0),
    }
}
