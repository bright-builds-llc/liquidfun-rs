//! Strict shared Phase 10 particle-group protocol contracts.

use liquidfun_test_protocol::{
    FloatBits, Phase10BehaviorLeaf, Phase10GroupDefinition, Phase10GroupDestination,
    Phase10GroupSource, Phase10Operation, Phase10PairSnapshot, Phase10Provenance,
    Phase10SemanticOutcome, Phase10Shape, Phase10StateObservation, Phase10TriadSnapshot,
    Phase10Witness, Phase10WitnessObservation, ScenarioId, TransformBits, Vec2Bits, WitnessRole,
};
use serde_json::{Value, json};

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

fn definition() -> Phase10GroupDefinition {
    Phase10GroupDefinition {
        system_id: id("system-a"),
        group_id: id("group-a"),
        member_ids: vec![id("particle-a"), id("particle-b")].into_boxed_slice(),
        source: Phase10GroupSource::Filled {
            shapes: vec![Phase10Shape::Circle {
                center: vector(0.0, 0.0),
                radius_bits: bits(1.0),
            }]
            .into_boxed_slice(),
        },
        destination: Phase10GroupDestination::New,
        particle_flags_bits: 1 << 3,
        group_flags_bits: 1,
        transform: TransformBits {
            position: vector(2.0, 3.0),
            angle_bits: bits(0.25),
        },
        linear_velocity: vector(4.0, 5.0),
        angular_velocity_bits: bits(0.5),
        color: [1, 2, 3, 4],
        strength_bits: bits(0.75),
        maybe_stride_bits: Some(bits(0.25)),
        lifetime_bits: bits(8.0),
    }
}

#[test]
fn semantic_group_definition_preserves_exact_bits_ids_and_source_order() {
    // Arrange
    let operation = Phase10Operation::CreateGroup {
        definition: definition(),
    };

    // Act
    let encoded = serde_json::to_value(&operation).expect("semantic operation should encode");
    let decoded: Phase10Operation =
        serde_json::from_value(encoded.clone()).expect("semantic operation should decode");

    // Assert
    assert_eq!(decoded, operation);
    assert_eq!(encoded["kind"], "create_group");
    assert_eq!(
        encoded["definition"]["member_ids"],
        json!(["particle-a", "particle-b"])
    );
    assert_eq!(encoded["definition"]["strength_bits"], bits(0.75).bits());
}

#[test]
fn semantic_full_triad_preserves_all_pinned_coefficients() {
    // Arrange
    let triad = Phase10TriadSnapshot {
        ordinal: 0,
        particle_a_id: id("particle-a"),
        particle_b_id: id("particle-b"),
        particle_c_id: id("particle-c"),
        flags_bits: 1 << 4,
        strength_bits: bits(0.5),
        pa: vector(-1.0, 0.0),
        pb: vector(1.0, 0.0),
        pc: vector(0.0, 1.0),
        ka_bits: bits(1.0),
        kb_bits: bits(2.0),
        kc_bits: bits(3.0),
        s_bits: bits(4.0),
    };

    // Act
    let encoded = serde_json::to_value(&triad).expect("triad should encode");
    let decoded: Phase10TriadSnapshot =
        serde_json::from_value(encoded).expect("triad should decode");

    // Assert
    assert_eq!(decoded, triad);
    assert_eq!(decoded.pa, vector(-1.0, 0.0));
    assert_eq!(decoded.pb, vector(1.0, 0.0));
    assert_eq!(decoded.pc, vector(0.0, 1.0));
    assert_eq!(decoded.ka_bits, bits(1.0));
    assert_eq!(decoded.kb_bits, bits(2.0));
    assert_eq!(decoded.kc_bits, bits(3.0));
    assert_eq!(decoded.s_bits, bits(4.0));
}

#[test]
fn semantic_witness_exposes_role_and_behavior_without_private_pass_data() {
    // Arrange
    let witness = Phase10Witness {
        ordinal: 0,
        behavior_leaf: Phase10BehaviorLeaf::Spring,
        role: WitnessRole::Interaction,
        observation: Phase10WitnessObservation::Topology {
            pair_count: 1,
            triad_count: 1,
        },
    };

    // Act
    let value = serde_json::to_value(&witness).expect("witness should encode");
    let object = value.as_object().expect("witness should be an object");

    // Assert
    assert_eq!(value["behavior_leaf"], "spring");
    assert_eq!(value["role"], "interaction");
    assert!(!object.contains_key("pass_id"));
    assert!(!object.contains_key("pass_trace"));
    assert!(!object.contains_key("pass_inventory"));
}

// Keep the complete public result surface linked into this integration target.
fn _semantic_surface_compile_guard(
    _provenance: Phase10Provenance,
    _outcome: Phase10SemanticOutcome,
    _state: Phase10StateObservation,
    _pair: Phase10PairSnapshot,
    _value: Value,
) {
}
