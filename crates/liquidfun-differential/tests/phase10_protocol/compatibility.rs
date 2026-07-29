#[test]
fn wire_phase9_request_and_result_variants_round_trip_unchanged() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(PHASE9_REQUEST, &limits)
        .expect("reviewed Phase 9 request should decode");
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("reviewed Phase 9 request should run");

    // Act
    let request_bytes =
        encode_jsonl(&request, &limits, RecordLimit::Input).expect("request should encode");
    let result_bytes =
        encode_jsonl(&result, &limits, RecordLimit::Output).expect("result should encode");
    let replay_request = decode_rigid_world_request_jsonl(&request_bytes, &limits)
        .expect("request replay should decode");
    let replay_result = decode_rigid_world_result_jsonl(&result_bytes, &limits)
        .expect("result replay should decode");

    // Assert
    assert_eq!(
        encode_jsonl(&replay_request, &limits, RecordLimit::Input),
        Ok(request_bytes)
    );
    assert_eq!(
        encode_jsonl(&replay_result, &limits, RecordLimit::Output),
        Ok(result_bytes)
    );
}

#[test]
fn wire_result_rejects_duplicate_witness_binding_order_and_nonfinite_observation() {
    // Arrange
    let witness = Phase10Witness {
        ordinal: 0,
        behavior_leaf: Phase10BehaviorLeaf::Spring,
        role: WitnessRole::Interaction,
        observation: Phase10WitnessObservation::Scalar {
            value_bits: bits(1.0),
        },
    };
    let mut duplicate = empty_observation();
    let Phase10Observation::State { state } = &mut duplicate;
    let mut repeated = witness.clone();
    repeated.ordinal = 1;
    state.witnesses = vec![witness.clone(), repeated].into_boxed_slice();
    let mut wrong_order = empty_observation();
    let Phase10Observation::State { state } = &mut wrong_order;
    let mut ordinal_one = witness.clone();
    ordinal_one.ordinal = 1;
    state.witnesses = vec![ordinal_one].into_boxed_slice();
    let mut nonfinite = empty_observation();
    let Phase10Observation::State { state } = &mut nonfinite;
    let mut nan_witness = witness;
    nan_witness.observation = Phase10WitnessObservation::Scalar {
        value_bits: FloatBits::new(f32::NAN.to_bits()),
    };
    state.witnesses = vec![nan_witness].into_boxed_slice();

    // Act
    let results =
        [duplicate, wrong_order, nonfinite].map(|observation| observation.validate_semantics());

    // Assert
    assert_eq!(
        results[0].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidWitness)
    );
    assert_eq!(
        results[1].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidOrdering)
    );
    assert_eq!(
        results[2].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidFloat)
    );
}

fn empty_observation() -> Phase10Observation {
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
            groups: Box::new([]),
            particles: Box::new([]),
            pairs: Box::new([]),
            triads: Box::new([]),
            particle_contacts: Box::new([]),
            body_contacts: Box::new([]),
            events: Box::new([]),
            witnesses: Box::new([]),
        },
    }
}

fn codec_kind(error: &RigidWorldDecodeError) -> Option<CodecErrorKind> {
    match error {
        RigidWorldDecodeError::Codec(error) => Some(error.kind()),
        RigidWorldDecodeError::Validation(_) => None,
    }
}

fn at_limit_definition(operation: &Phase10Operation) -> Phase10GroupDefinition {
    let Phase10Operation::CreateGroup { definition } = operation else {
        panic!("test operation should create a group");
    };
    definition.clone()
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
