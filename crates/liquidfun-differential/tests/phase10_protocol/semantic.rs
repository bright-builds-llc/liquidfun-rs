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
