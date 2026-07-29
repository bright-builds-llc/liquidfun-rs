use super::{
    BoundedString, BoundedVec, Deserialize, FloatBits, HarnessFailureKind, HarnessLimits,
    MAXIMUM_ID_BYTES, MathProbeHorizon, MathProbeOperation, MathProbePolicyPath, ProtocolVersion,
    RecordLimit, RequestId, ScenarioId, Serialize, TraceDecodeError, TraceValidationError,
    decode_jsonl,
};

/// IEEE-754 binary32 class reported beside every exact probe result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs, reason = "closed wire enum names are self-describing")]
pub enum MathProbeFloatClass {
    Zero,
    Subnormal,
    Normal,
    Infinite,
    Nan,
}

/// Closed names for ordered scalar components emitted by a math probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs, reason = "closed wire enum names are self-describing")]
pub enum MathProbeValueField {
    Value,
    X,
    Y,
    Z,
    Length,
    Sine,
    Cosine,
    PositionX,
    PositionY,
    Angle,
    InitialCenterX,
    InitialCenterY,
    InitialAngle,
    InitialFraction,
    LeftAssociated,
    RightAssociated,
    EvenMidpoint,
    OddMidpoint,
}

/// One exact float result with explicit class and sign metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MathProbeValue {
    field: MathProbeValueField,
    bits: FloatBits,
    class: MathProbeFloatClass,
    negative: bool,
}

#[allow(
    missing_docs,
    reason = "private harness accessors mirror named wire fields"
)]
impl MathProbeValue {
    #[must_use]
    pub fn new(field: MathProbeValueField, bits: FloatBits) -> Self {
        let raw = bits.bits();
        let exponent = raw & 0x7f80_0000;
        let fraction = raw & 0x007f_ffff;
        let class = match (exponent, fraction) {
            (0, 0) => MathProbeFloatClass::Zero,
            (0, _) => MathProbeFloatClass::Subnormal,
            (0x7f80_0000, 0) => MathProbeFloatClass::Infinite,
            (0x7f80_0000, _) => MathProbeFloatClass::Nan,
            _ => MathProbeFloatClass::Normal,
        };
        Self {
            field,
            bits,
            class,
            negative: raw & 0x8000_0000 != 0,
        }
    }

    #[must_use]
    pub const fn field(self) -> MathProbeValueField {
        self.field
    }
    #[must_use]
    pub const fn bits(self) -> FloatBits {
        self.bits
    }
    #[must_use]
    pub const fn class(self) -> MathProbeFloatClass {
        self.class
    }
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.negative
    }
}

/// Closed discrete branches and predicates reported by pure probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs, reason = "closed wire enum names are self-describing")]
pub enum MathProbeDiscreteField {
    Predicate,
    NonZeroDeterminant,
    Normalized,
    Advanced,
}

/// One exact boolean probe result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MathProbeDiscrete {
    field: MathProbeDiscreteField,
    value: bool,
}

#[allow(
    missing_docs,
    reason = "private harness accessors mirror named wire fields"
)]
impl MathProbeDiscrete {
    #[must_use]
    pub const fn new(field: MathProbeDiscreteField, value: bool) -> Self {
        Self { field, value }
    }
    #[must_use]
    pub const fn field(self) -> MathProbeDiscreteField {
        self.field
    }
    #[must_use]
    pub const fn value(self) -> bool {
        self.value
    }
}

/// Ordered, policy-bound output of one native or reference math case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MathProbeResult {
    case_id: Box<str>,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    horizon: MathProbeHorizon,
    values: Box<[MathProbeValue]>,
    discrete: Box<[MathProbeDiscrete]>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMathProbeResult {
    case_id: BoundedString<MAXIMUM_ID_BYTES>,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    horizon: MathProbeHorizon,
    values: BoundedVec<MathProbeValue, 32>,
    discrete: BoundedVec<MathProbeDiscrete, 8>,
}

/// Strictly decodes one bounded, internally consistent math-probe result record.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, resource, identifier, or float-metadata failure.
pub fn decode_math_probe_result_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<MathProbeResult, TraceDecodeError> {
    let raw = decode_jsonl::<RawMathProbeResult>(bytes, limits, RecordLimit::Output)?;
    let case_id = raw.case_id.into_string();
    ScenarioId::new(case_id.clone()).map_err(|_| {
        TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "math probe result case ID is invalid",
        )
    })?;
    let values = raw.values.into_vec();
    for value in &values {
        let bits = value.bits();
        if value.class() != classify_math_probe_float(bits)
            || value.is_negative() != (bits.bits() & 0x8000_0000 != 0)
        {
            return Err(TraceValidationError::new(
                HarnessFailureKind::SequenceViolation,
                "math probe float metadata does not match authoritative bits",
            )
            .into());
        }
    }
    Ok(MathProbeResult::new(
        case_id,
        raw.operation,
        raw.policy_path,
        raw.horizon,
        values,
        raw.discrete.into_vec(),
    ))
}

fn classify_math_probe_float(bits: FloatBits) -> MathProbeFloatClass {
    let value = bits.to_f32();
    if value.is_nan() {
        MathProbeFloatClass::Nan
    } else if value.is_infinite() {
        MathProbeFloatClass::Infinite
    } else if value == 0.0 {
        MathProbeFloatClass::Zero
    } else if value.is_subnormal() {
        MathProbeFloatClass::Subnormal
    } else {
        MathProbeFloatClass::Normal
    }
}

/// Validated terminal record for one one-shot math-probe request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathProbeEnd {
    request_id: RequestId,
    result_count: u32,
    reset_epoch: u64,
}

impl MathProbeEnd {
    /// Returns the request identity echoed by the adapter.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the exact number of preceding result records.
    #[must_use]
    pub const fn result_count(&self) -> u32 {
        self.result_count
    }

    /// Returns the independently advanced reset epoch.
    #[must_use]
    pub const fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMathProbeEnd {
    protocol_version: ProtocolVersion,
    record_kind: MathProbeEndKind,
    request_id: BoundedString<MAXIMUM_ID_BYTES>,
    result_count: u32,
    reset_epoch: u64,
    reset_verified: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MathProbeEndKind {
    MathProbeEnd,
}

/// Strictly decodes one bounded terminal math-probe record.
///
/// # Errors
///
/// Returns [`TraceDecodeError`] for framing, resource, identity, or reset-proof failure.
pub fn decode_math_probe_end_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<MathProbeEnd, TraceDecodeError> {
    let RawMathProbeEnd {
        protocol_version,
        record_kind,
        request_id,
        result_count,
        reset_epoch,
        reset_verified,
    } = decode_jsonl::<RawMathProbeEnd>(bytes, limits, RecordLimit::Output)?;
    let _ = protocol_version;
    let _ = record_kind;
    let request_id = RequestId::new(request_id.into_string()).map_err(|_| {
        TraceValidationError::new(
            HarnessFailureKind::SequenceViolation,
            "math probe end request ID is invalid",
        )
    })?;
    if !reset_verified {
        return Err(TraceValidationError::new(
            HarnessFailureKind::AdapterResetFailure,
            "math probe end did not prove reset",
        )
        .into());
    }
    Ok(MathProbeEnd {
        request_id,
        result_count,
        reset_epoch,
    })
}

#[allow(
    missing_docs,
    reason = "private harness accessors mirror named wire fields"
)]
impl MathProbeResult {
    #[must_use]
    pub fn new(
        case_id: impl Into<Box<str>>,
        operation: MathProbeOperation,
        policy_path: MathProbePolicyPath,
        horizon: MathProbeHorizon,
        values: Vec<MathProbeValue>,
        discrete: Vec<MathProbeDiscrete>,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            operation,
            policy_path,
            horizon,
            values: values.into_boxed_slice(),
            discrete: discrete.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
    #[must_use]
    pub const fn operation(&self) -> MathProbeOperation {
        self.operation
    }
    #[must_use]
    pub const fn policy_path(&self) -> MathProbePolicyPath {
        self.policy_path
    }
    #[must_use]
    pub const fn horizon(&self) -> MathProbeHorizon {
        self.horizon
    }
    #[must_use]
    pub fn values(&self) -> &[MathProbeValue] {
        &self.values
    }
    #[must_use]
    pub fn discrete(&self) -> &[MathProbeDiscrete] {
        &self.discrete
    }
}
