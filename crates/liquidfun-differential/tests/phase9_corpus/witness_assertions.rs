#[allow(
    clippy::too_many_lines,
    reason = "one closed assertion match keeps every inherited witness class auditable"
)]
fn assert_witness(request: &Value, result: &Value, witness: &Phase9WitnessBinding) {
    let branch = witness.branch_id.as_str();
    if witness.semantic_assertion.requires_case_evidence() {
        assert!(assert_contract_witness(request, result, branch));
        return;
    }
    let observation = observation_for_witness(request, result, witness);
    assert_eq!(observation["kind"], "particle", "{branch}");
    assert_eq!(
        observation["observation"]["kind"],
        serde_json::to_value(witness.observation_kind).expect("observation kind should serialize"),
        "{branch}"
    );
    match &witness.semantic_assertion {
        Phase9SemanticAssertion::ObservedSemantic { branch_id } => {
            assert_observed_semantic(request, result, branch_id.as_str());
        }
        Phase9SemanticAssertion::FiniteLifetimeExpired { particle_id } => {
            assert!(
                !observation["observation"]["particle_ids"]
                    .as_array()
                    .expect("system particle IDs")
                    .iter()
                    .any(|id| id == particle_id.as_str()),
                "{} must have expired",
                particle_id.as_str()
            );
        }
        Phase9SemanticAssertion::InfiniteLifetimeSurvives { particle_id } => {
            assert!(
                observation["observation"]["particle_ids"]
                    .as_array()
                    .expect("system particle IDs")
                    .iter()
                    .any(|id| id == particle_id.as_str()),
                "{} must survive",
                particle_id.as_str()
            );
        }
        Phase9SemanticAssertion::EqualExpirationOrder { particle_ids } => {
            assert_eq!(
                particle_declaration(request, particle_ids[0].as_str())["lifetime_bits"],
                particle_declaration(request, particle_ids[1].as_str())["lifetime_bits"]
            );
            assert_eq!(
                observation["observation"]["occurrence"]["maybe_particle_id"],
                particle_ids[1].as_str(),
                "equal expirations must evict newest-first"
            );
        }
        Phase9SemanticAssertion::StrictContactCardinality {
            enabled,
            contact_count,
        } => {
            let system_id = observation["observation"]["statistics"]["maybe_system_id"]
                .as_str()
                .expect("statistics system ID");
            assert_eq!(
                system_declaration(request, system_id)["strict_contact_check"],
                *enabled
            );
            assert_eq!(
                observation["observation"]["statistics"]["body_contact_count"],
                *contact_count
            );
        }
        Phase9SemanticAssertion::ListenerEventEffect {
            enabled,
            event_count,
        } => {
            let occurrences = phase9_checkpoint(result, "phase9-corpus")["observations"]
                .as_array()
                .expect("Phase 9 observations")
                .iter()
                .filter(|candidate| {
                    candidate["observation"]["kind"] == "lifecycle"
                        && candidate["observation"]["occurrence"]["kind"] == "contact_created"
                        && (*enabled
                            || candidate["observation"]["occurrence"]["maybe_particle_id"]
                                == "phase9-capacity"
                            || candidate["observation"]["occurrence"]["maybe_other_particle_id"]
                                == "phase9-capacity")
                })
                .count();
            assert_eq!(
                u32::try_from(occurrences).expect("event count fits"),
                *event_count
            );
        }
        Phase9SemanticAssertion::FilterContactEffect {
            enabled,
            contact_count,
        } => {
            let expected_system = if *enabled {
                "phase9-growable"
            } else {
                "phase9-fixed-paused"
            };
            assert_eq!(
                observation["observation"]["statistics"]["maybe_system_id"],
                expected_system
            );
            assert_eq!(
                observation["observation"]["statistics"]["particle_contact_count"],
                *contact_count
            );
        }
        Phase9SemanticAssertion::CollisionEnergyPositiveFinite { minimum_bits } => {
            let bits = u32::try_from(
                observation["observation"]["statistics"]["collision_energy_bits"]
                    .as_u64()
                    .expect("collision-energy bits"),
            )
            .expect("collision-energy bits fit");
            let energy = f32::from_bits(bits);
            assert!(energy.is_finite());
            assert!(energy >= minimum_bits.to_f32());
        }
        Phase9SemanticAssertion::StuckCandidatesNonempty { particle_ids } => {
            let stuck = observation["observation"]["statistics"]["stuck_particle_ids"]
                .as_array()
                .expect("stuck particle IDs");
            assert!(!stuck.is_empty());
            for particle_id in particle_ids {
                assert!(stuck.iter().any(|id| id == particle_id.as_str()));
            }
        }
        Phase9SemanticAssertion::ReplayResultDigestEquality
        | Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
        | Phase9SemanticAssertion::DeliberateFirstDivergence
        | Phase9SemanticAssertion::D0RepeatedResultDigestEquality
        | Phase9SemanticAssertion::DebugReleaseResultDigestEquality => {}
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "proof construction binds all independently produced payloads in one audited step"
)]
fn build_cross_run_proofs(
    root: &Path,
    case_id: &str,
    selected_oracle: &OracleExecutable,
    revision: &str,
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    _oracle: &RigidWorldResultRecord,
    request_bytes: &[u8],
    native_bytes: &[u8],
    oracle_bytes: &[u8],
    bindings: &[Phase9WitnessBinding],
) -> (Vec<Phase9CrossRunProofRecord>, BTreeMap<String, Vec<u8>>) {
    let case_bindings = bindings
        .iter()
        .filter(|binding| binding.semantic_assertion.requires_case_evidence())
        .collect::<Vec<_>>();
    if case_bindings.is_empty() {
        return (Vec::new(), BTreeMap::new());
    }
    let debug =
        OracleExecutable::resolve(root, OraclePreset::Debug).expect("debug oracle must be built");
    let release = OracleExecutable::resolve(root, OraclePreset::Release)
        .expect("release oracle must be built");
    let replay_native =
        NativeRigidWorldExecutor::execute(request).expect("native result replay must execute");
    let replay_oracle = execute_rigid_world_process(selected_oracle, request, revision)
        .expect("oracle result replay must execute");
    let debug_oracle = execute_rigid_world_process(&debug, request, revision)
        .expect("independent debug result must execute");
    let release_oracle = execute_rigid_world_process(&release, request, revision)
        .expect("independent release result must execute");
    let minimized = mutated_phase9_result(native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        body["active"] = json!(!body["active"].as_bool().expect("body active"));
    });
    let copied = mutated_phase9_result(native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        body["active"] = json!(!body["active"].as_bool().expect("body active"));
        let fixture = first_checkpoint_member_mut(value, "fixtures");
        fixture["sensor"] = json!(!fixture["sensor"].as_bool().expect("fixture sensor"));
    });
    let minimized_report = expected_retained_mismatch(request, native, &minimized);
    let copied_report = expected_retained_mismatch(request, native, &copied);
    let base = format!("cases/{case_id}/proofs");
    let replay_native_path = format!("{base}/replay-native.json");
    let replay_oracle_path = format!("{base}/replay-oracle.json");
    let minimized_path = format!("{base}/minimized.json");
    let copied_path = format!("{base}/copied.json");
    let debug_path = format!("{base}/debug.json");
    let release_path = format!("{base}/release.json");
    let mut payloads = BTreeMap::from([
        (
            replay_native_path.clone(),
            serde_json::to_vec(&replay_native).expect("native replay bytes"),
        ),
        (
            replay_oracle_path.clone(),
            serde_json::to_vec(replay_oracle.result()).expect("oracle replay bytes"),
        ),
        (
            minimized_path.clone(),
            serde_json::to_vec(&minimized).expect("minimized result bytes"),
        ),
        (
            copied_path.clone(),
            serde_json::to_vec(&copied).expect("copied result bytes"),
        ),
        (
            debug_path.clone(),
            serde_json::to_vec(debug_oracle.result()).expect("debug result bytes"),
        ),
        (
            release_path.clone(),
            serde_json::to_vec(release_oracle.result()).expect("release result bytes"),
        ),
    ]);
    let payload_ref = |path: &String| Phase9EvidencePayloadRef {
        path: path.clone().into(),
        sha256: Sha256Hex::new(sha256(payloads.get(path).expect("proof payload")))
            .expect("computed digest"),
    };
    let mismatch = |path: &String, report: &RigidMismatchReport| Phase9EvidenceMismatch {
        result: payload_ref(path),
        signature_sha256: report.signature().signature_sha256().clone(),
        semantic_path: report.semantic_path().into(),
    };
    let request_sha256 = Sha256Hex::new(sha256(request_bytes)).expect("computed request digest");
    let native_sha256 = Sha256Hex::new(sha256(native_bytes)).expect("computed native digest");
    let oracle_sha256 = Sha256Hex::new(sha256(oracle_bytes)).expect("computed oracle digest");
    let records = case_bindings
        .into_iter()
        .map(|binding| {
            let proof = match &binding.semantic_assertion {
                Phase9SemanticAssertion::ReplayResultDigestEquality => {
                    Phase9CrossRunProof::ReplayResultDigestEquality {
                        replay_native: payload_ref(&replay_native_path),
                        replay_oracle: payload_ref(&replay_oracle_path),
                    }
                }
                Phase9SemanticAssertion::MinimizedFailureSignaturePreservation => {
                    Phase9CrossRunProof::MinimizedFailureSignaturePreservation {
                        minimized: mismatch(&minimized_path, &minimized_report),
                        copied: mismatch(&copied_path, &copied_report),
                    }
                }
                Phase9SemanticAssertion::DeliberateFirstDivergence => {
                    Phase9CrossRunProof::DeliberateFirstDivergence {
                        minimized: mismatch(&minimized_path, &minimized_report),
                        copied: mismatch(&copied_path, &copied_report),
                    }
                }
                Phase9SemanticAssertion::D0RepeatedResultDigestEquality => {
                    Phase9CrossRunProof::D0RepeatedResultDigestEquality {
                        repeated_native: payload_ref(&replay_native_path),
                        repeated_oracle: payload_ref(&replay_oracle_path),
                    }
                }
                Phase9SemanticAssertion::DebugReleaseResultDigestEquality => {
                    Phase9CrossRunProof::DebugReleaseResultDigestEquality {
                        debug_oracle: payload_ref(&debug_path),
                        release_oracle: payload_ref(&release_path),
                    }
                }
                _ => unreachable!("filtered case evidence binding"),
            };
            Phase9CrossRunProofRecord {
                branch_id: binding.branch_id.clone(),
                request_sha256: request_sha256.clone(),
                native_result_sha256: native_sha256.clone(),
                oracle_result_sha256: oracle_sha256.clone(),
                proof,
            }
        })
        .collect();
    (records, std::mem::take(&mut payloads))
}
