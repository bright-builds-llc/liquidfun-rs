use super::{
    BoundedString, BoundedVec, BuildIdentity, BuildIdentityFields, CheckpointId, CheckpointRecord,
    Deserialize, EngineKind, FloatBits, HandshakeRecord, HarnessFailureKind, HarnessLimits,
    MAXIMUM_ID_BYTES, MAXIMUM_STRING_BYTES, MAXIMUM_SUPPORTED_VERSIONS, Phase4BuildIdentityFields,
    ProtocolVersion, RecordLimit, RequestId, ScenarioId, ScenarioSchemaVersion, ScenarioSource,
    Sha256Hex, ToleranceProfileVersion, TraceBegin, TraceBeginKind, TraceDecodeError, TraceEnd,
    TraceRecord, TraceSchemaVersion, TraceValidationError, WorldCounts, decode_jsonl,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHandshakeRecord {
    protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: HandshakeKind,
    supported_scenario_versions: BoundedVec<ScenarioSchemaVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    supported_trace_versions: BoundedVec<TraceSchemaVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    supported_tolerance_versions: BoundedVec<ToleranceProfileVersion, MAXIMUM_SUPPORTED_VERSIONS>,
    build_identity: RawBuildIdentity,
    identity_sha256: Sha256Hex,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum HandshakeKind {
    Handshake,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuildIdentity {
    oracle_revision: BoundedString<MAXIMUM_STRING_BYTES>,
    adapter_revision: BoundedString<MAXIMUM_STRING_BYTES>,
    adapter_content_sha256: BoundedString<MAXIMUM_STRING_BYTES>,
    cmake_preset: BoundedString<MAXIMUM_STRING_BYTES>,
    compiler_id: BoundedString<MAXIMUM_STRING_BYTES>,
    compiler_version: BoundedString<MAXIMUM_STRING_BYTES>,
    target: BoundedString<MAXIMUM_STRING_BYTES>,
    build_type: BoundedString<MAXIMUM_STRING_BYTES>,
    effective_compile_flags: BoundedString<MAXIMUM_STRING_BYTES>,
    effective_link_flags: BoundedString<MAXIMUM_STRING_BYTES>,
    sanitizer_mode: BoundedString<MAXIMUM_STRING_BYTES>,
    compile_command_sha256: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_triple: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_cpu: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_features: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    sdk_or_sysroot: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    optimization: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    fp_model: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    fp_contract: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    denormal_mode: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    feature_set: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    os: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    libc: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    libm: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    rounding_mode: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    gradual_underflow: Option<bool>,
}

/// Strictly decodes and independently recomputes one startup handshake.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, shape, limit, or provenance failure.
pub fn decode_handshake_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<HandshakeRecord, TraceDecodeError> {
    let raw = decode_jsonl::<RawHandshakeRecord>(bytes, limits, RecordLimit::Output)?;
    let RawHandshakeRecord {
        protocol_version,
        _record_kind: _,
        supported_scenario_versions,
        supported_trace_versions,
        supported_tolerance_versions,
        build_identity: raw_identity,
        identity_sha256,
    } = raw;
    let compiler_id = raw_identity.compiler_id.into_string();
    let compiler_version = raw_identity.compiler_version.into_string();
    let maybe_phase4 = decode_raw_phase4_identity(
        &compiler_id,
        &compiler_version,
        raw_identity.compile_command_sha256,
        raw_identity.target_triple,
        raw_identity.target_cpu,
        raw_identity.target_features,
        raw_identity.sdk_or_sysroot,
        raw_identity.optimization,
        raw_identity.fp_model,
        raw_identity.fp_contract,
        raw_identity.denormal_mode,
        raw_identity.feature_set,
        raw_identity.os,
        raw_identity.libc,
        raw_identity.libm,
        raw_identity.rounding_mode,
        raw_identity.gradual_underflow,
    )?;
    let mut fields = BuildIdentityFields::new(
        raw_identity.oracle_revision.into_string(),
        raw_identity.adapter_revision.into_string(),
        raw_identity.adapter_content_sha256.into_string(),
        raw_identity.cmake_preset.into_string(),
        compiler_id,
        compiler_version,
        raw_identity.target.into_string(),
        raw_identity.build_type.into_string(),
        raw_identity.effective_compile_flags.into_string(),
        raw_identity.effective_link_flags.into_string(),
        raw_identity.sanitizer_mode.into_string(),
    );
    if let Some(phase4) = maybe_phase4 {
        fields = fields.with_phase4(phase4);
    }
    let build_identity =
        BuildIdentity::from_reported(fields, &identity_sha256).map_err(|error| {
            TraceValidationError::new(HarnessFailureKind::WrongProvenance, error.to_string())
        })?;
    Ok(HandshakeRecord {
        protocol_version,
        supported_scenario_versions: supported_scenario_versions.into_vec().into_boxed_slice(),
        supported_trace_versions: supported_trace_versions.into_vec().into_boxed_slice(),
        supported_tolerance_versions: supported_tolerance_versions.into_vec().into_boxed_slice(),
        build_identity,
    })
}

#[allow(
    clippy::similar_names,
    clippy::too_many_arguments,
    reason = "the fixed wire vocabulary deliberately distinguishes libc from libm"
)]
fn decode_raw_phase4_identity(
    compiler_id: &str,
    compiler_version: &str,
    compile_command_sha256: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_triple: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_cpu: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    target_features: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    sdk_or_sysroot: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    optimization: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    fp_model: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    fp_contract: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    denormal_mode: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    feature_set: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    os: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    libc: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    libm: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    rounding_mode: Option<BoundedString<MAXIMUM_STRING_BYTES>>,
    gradual_underflow: Option<bool>,
) -> Result<Option<Phase4BuildIdentityFields>, TraceValidationError> {
    let fields_present = [
        compile_command_sha256.is_some(),
        target_triple.is_some(),
        target_cpu.is_some(),
        target_features.is_some(),
        sdk_or_sysroot.is_some(),
        optimization.is_some(),
        fp_model.is_some(),
        fp_contract.is_some(),
        denormal_mode.is_some(),
        feature_set.is_some(),
        os.is_some(),
        libc.is_some(),
        libm.is_some(),
        rounding_mode.is_some(),
        gradual_underflow.is_some(),
    ];
    if fields_present.iter().all(|present| !present) {
        return Ok(None);
    }
    if !fields_present.iter().all(|present| *present) {
        return Err(TraceValidationError::new(
            HarnessFailureKind::WrongProvenance,
            "Phase 4 build identity fields must be present together",
        ));
    }
    let text = |value: Option<BoundedString<MAXIMUM_STRING_BYTES>>| {
        value.map(BoundedString::into_string).ok_or_else(|| {
            TraceValidationError::new(
                HarnessFailureKind::WrongProvenance,
                "Phase 4 build identity field is missing",
            )
        })
    };
    Ok(Some(Phase4BuildIdentityFields::new(
        text(compile_command_sha256)?,
        compiler_id,
        compiler_version,
        text(target_triple)?,
        text(target_cpu)?,
        text(target_features)?,
        text(sdk_or_sysroot)?,
        text(optimization)?,
        text(fp_model)?,
        text(fp_contract)?,
        text(denormal_mode)?,
        text(feature_set)?,
        text(os)?,
        text(libc)?,
        text(libm)?,
        text(rounding_mode)?,
        gradual_underflow.ok_or_else(|| {
            TraceValidationError::new(
                HarnessFailureKind::WrongProvenance,
                "Phase 4 gradual-underflow witness is missing",
            )
        })?,
    )))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "record_kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawTraceRecord {
    TraceBegin {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        trace_schema_version: TraceSchemaVersion,
        scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
        scenario_sha256: Sha256Hex,
        source: RawTraceSource,
        tolerance_profile_version: ToleranceProfileVersion,
        tolerance_profile_sha256: Sha256Hex,
        engine_kind: EngineKind,
        identity_sha256: Sha256Hex,
    },
    Checkpoint {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        checkpoint_id: BoundedString<MAXIMUM_ID_BYTES>,
        ordinal: u32,
        phase: BoundedString<MAXIMUM_STRING_BYTES>,
        simulation_time_bits: FloatBits,
        world_counts: WorldCounts,
        identity_sha256: Sha256Hex,
    },
    TraceEnd {
        protocol_version: ProtocolVersion,
        request_id: BoundedString<MAXIMUM_ID_BYTES>,
        checkpoint_count: u32,
        trace_payload_sha256: Sha256Hex,
        reset_epoch: u64,
        reset_verified: bool,
        identity_sha256: Sha256Hex,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawTraceSource {
    Named {
        name: BoundedString<MAXIMUM_STRING_BYTES>,
    },
    Seeded {
        generator_id: BoundedString<MAXIMUM_STRING_BYTES>,
        generator_version: u32,
        seed: u64,
    },
}

/// Strictly decodes one streamed trace record into its closed typed variant.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, shape, limit, ID, or phase failure.
pub fn decode_trace_record_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<TraceRecord, TraceDecodeError> {
    let raw = decode_jsonl::<RawTraceRecord>(bytes, limits, RecordLimit::Output)?;
    match raw {
        RawTraceRecord::TraceBegin {
            protocol_version,
            request_id,
            trace_schema_version,
            scenario_id,
            scenario_sha256,
            source,
            tolerance_profile_version,
            tolerance_profile_sha256,
            engine_kind,
            identity_sha256,
        } => Ok(TraceRecord::Begin(TraceBegin {
            protocol_version,
            record_kind: TraceBeginKind::TraceBegin,
            request_id: parse_request_id(request_id)?,
            trace_schema_version,
            scenario_id: ScenarioId::new(scenario_id.into_string()).map_err(|error| {
                TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
            })?,
            scenario_sha256,
            source: convert_source(source)?,
            tolerance_profile_version,
            tolerance_profile_sha256,
            engine_kind,
            identity_sha256,
        })),
        RawTraceRecord::Checkpoint {
            protocol_version,
            request_id,
            checkpoint_id,
            ordinal,
            phase,
            simulation_time_bits,
            world_counts,
            identity_sha256,
        } => {
            let mut checkpoint = CheckpointRecord::new(
                parse_request_id(request_id)?,
                CheckpointId::new(checkpoint_id.into_string()).map_err(|error| {
                    TraceValidationError::new(
                        HarnessFailureKind::MalformedRecord,
                        error.to_string(),
                    )
                })?,
                ordinal,
                phase.into_string(),
                simulation_time_bits,
                world_counts,
                identity_sha256,
            )?;
            checkpoint.protocol_version = protocol_version;
            Ok(TraceRecord::Checkpoint(checkpoint))
        }
        RawTraceRecord::TraceEnd {
            protocol_version,
            request_id,
            checkpoint_count,
            trace_payload_sha256,
            reset_epoch,
            reset_verified,
            identity_sha256,
        } => {
            let mut end = TraceEnd::new(
                parse_request_id(request_id)?,
                checkpoint_count,
                trace_payload_sha256,
                reset_epoch,
                reset_verified,
                identity_sha256,
            );
            end.protocol_version = protocol_version;
            Ok(TraceRecord::End(end))
        }
    }
}

fn parse_request_id(
    raw: BoundedString<MAXIMUM_ID_BYTES>,
) -> Result<RequestId, TraceValidationError> {
    RequestId::new(raw.into_string()).map_err(|error| {
        TraceValidationError::new(HarnessFailureKind::MalformedRecord, error.to_string())
    })
}

fn convert_source(raw: RawTraceSource) -> Result<ScenarioSource, TraceValidationError> {
    match raw {
        RawTraceSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(invalid_trace_source());
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawTraceSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(invalid_trace_source());
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

fn invalid_trace_source() -> TraceValidationError {
    TraceValidationError::new(
        HarnessFailureKind::MalformedRecord,
        "trace source identity must be nonempty and versioned",
    )
}
