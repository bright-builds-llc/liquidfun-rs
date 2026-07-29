use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    BenchmarkWireError, BenchmarkWireErrorKind, MAXIMUM_MEASURED_HORIZON, MAXIMUM_RESOLVED_BYTES,
    benchmark_policy_sha256, validation,
};
use crate::performance::{
    PerformancePolicy, PerformanceSizePoint, PerformanceWorkloadKind, ScalarOptimizationMode,
};
use crate::{
    HarnessLimits, ProtocolVersion, RecordLimit, RequestId, RunSettings, Sha256Hex,
    codec::BoundedVec, decode_jsonl,
};

/// Identity that must be identical across one request and its engine result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIdentity {
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    settings: RunSettings,
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    optimization_mode: ScalarOptimizationMode,
    warmup_count: u8,
    measured_horizon: u32,
    sample_ordinal: u16,
    policy_sha256: Sha256Hex,
    profile_enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBenchmarkRunIdentity {
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    settings: RunSettings,
    workload: PerformanceWorkloadKind,
    size_point: PerformanceSizePoint,
    optimization_mode: ScalarOptimizationMode,
    warmup_count: u8,
    measured_horizon: u32,
    sample_ordinal: u16,
    policy_sha256: Sha256Hex,
    profile_enabled: bool,
}

impl<'de> Deserialize<'de> for BenchmarkRunIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawBenchmarkRunIdentity::deserialize(deserializer)?;
        Self::new(
            raw.request_id,
            raw.resolved_sha256,
            raw.settings,
            raw.workload,
            raw.size_point,
            raw.optimization_mode,
            raw.warmup_count,
            raw.measured_horizon,
            raw.sample_ordinal,
            raw.policy_sha256,
            raw.profile_enabled,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl BenchmarkRunIdentity {
    /// Validates every identity and reviewed execution-policy field.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkWireError`] when a policy, horizon, or ordinal bound is invalid.
    #[allow(
        clippy::too_many_arguments,
        reason = "all eleven cross-engine identity fields are intentionally mandatory"
    )]
    pub fn new(
        request_id: RequestId,
        resolved_sha256: Sha256Hex,
        settings: RunSettings,
        workload: PerformanceWorkloadKind,
        size_point: PerformanceSizePoint,
        optimization_mode: ScalarOptimizationMode,
        warmup_count: u8,
        measured_horizon: u32,
        sample_ordinal: u16,
        policy_sha256: Sha256Hex,
        profile_enabled: bool,
    ) -> Result<Self, BenchmarkWireError> {
        let policy = PerformancePolicy::reviewed_v1();
        if warmup_count != policy.warmup_runs() {
            return Err(validation(BenchmarkWireErrorKind::InvalidWarmupCount));
        }
        if !(1..=MAXIMUM_MEASURED_HORIZON).contains(&measured_horizon) {
            return Err(validation(BenchmarkWireErrorKind::InvalidMeasuredHorizon));
        }
        if !(1..=policy.samples_per_engine()).contains(&sample_ordinal) {
            return Err(validation(BenchmarkWireErrorKind::InvalidSampleOrdinal));
        }
        if policy_sha256 != benchmark_policy_sha256()? {
            return Err(validation(BenchmarkWireErrorKind::PolicyMismatch));
        }
        Ok(Self {
            request_id,
            resolved_sha256,
            settings,
            workload,
            size_point,
            optimization_mode,
            warmup_count,
            measured_horizon,
            sample_ordinal,
            policy_sha256,
            profile_enabled,
        })
    }

    /// Returns the stable request ID.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the exact resolved-byte hash.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns the exact timestep and solver settings.
    #[must_use]
    pub const fn settings(&self) -> RunSettings {
        self.settings
    }

    /// Returns the reviewed workload.
    #[must_use]
    pub const fn workload(&self) -> PerformanceWorkloadKind {
        self.workload
    }

    /// Returns the reviewed cardinality point.
    #[must_use]
    pub const fn size_point(&self) -> PerformanceSizePoint {
        self.size_point
    }

    /// Returns the scalar optimization mode.
    #[must_use]
    pub const fn optimization_mode(&self) -> ScalarOptimizationMode {
        self.optimization_mode
    }

    /// Returns the excluded warm-up count.
    #[must_use]
    pub const fn warmup_count(&self) -> u8 {
        self.warmup_count
    }

    /// Returns the fixed logical measured horizon.
    #[must_use]
    pub const fn measured_horizon(&self) -> u32 {
        self.measured_horizon
    }

    /// Returns the one-based sample ordinal.
    #[must_use]
    pub const fn sample_ordinal(&self) -> u16 {
        self.sample_ordinal
    }

    /// Returns the reviewed measurement-policy hash.
    #[must_use]
    pub const fn policy_sha256(&self) -> &Sha256Hex {
        &self.policy_sha256
    }

    /// Reports whether optional diagnostic profiling was enabled.
    #[must_use]
    pub const fn profile_enabled(&self) -> bool {
        self.profile_enabled
    }
}

/// Strict benchmark request carrying the exact resolved scenario bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkRunRequest {
    identity: BenchmarkRunIdentity,
    resolved_bytes: Box<[u8]>,
}

impl BenchmarkRunRequest {
    /// Validates the exact resolved bytes against the shared run identity.
    ///
    /// # Errors
    ///
    /// Returns [`BenchmarkWireError`] when the bytes are oversized or their hash differs.
    pub fn new(
        identity: BenchmarkRunIdentity,
        resolved_bytes: Vec<u8>,
    ) -> Result<Self, BenchmarkWireError> {
        if resolved_bytes.len() > MAXIMUM_RESOLVED_BYTES {
            return Err(validation(BenchmarkWireErrorKind::ResolvedBytesTooLarge));
        }
        let actual = Sha256Hex::from_digest(Sha256::digest(&resolved_bytes).into());
        if actual != *identity.resolved_sha256() {
            return Err(validation(BenchmarkWireErrorKind::ResolvedHashMismatch));
        }
        Ok(Self {
            identity,
            resolved_bytes: resolved_bytes.into_boxed_slice(),
        })
    }

    /// Returns the shared request/result identity.
    #[must_use]
    pub const fn identity(&self) -> &BenchmarkRunIdentity {
        &self.identity
    }

    /// Returns the exact bytes both engines must execute.
    #[must_use]
    pub const fn resolved_bytes(&self) -> &[u8] {
        &self.resolved_bytes
    }
}

#[derive(Serialize)]
struct BenchmarkRunRequestRef<'a> {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    identity: &'a BenchmarkRunIdentity,
    resolved_bytes: &'a [u8],
}

impl Serialize for BenchmarkRunRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        BenchmarkRunRequestRef {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: "benchmark_run_request",
            identity: &self.identity,
            resolved_bytes: &self.resolved_bytes,
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBenchmarkRunRequest {
    #[serde(rename = "protocol_version")]
    _protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: BenchmarkRunRequestRecordKind,
    identity: BenchmarkRunIdentity,
    resolved_bytes: BoundedVec<u8, MAXIMUM_RESOLVED_BYTES>,
}

#[derive(Deserialize)]
enum BenchmarkRunRequestRecordKind {
    #[serde(rename = "benchmark_run_request")]
    BenchmarkRunRequest,
}

pub(super) fn decode(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<BenchmarkRunRequest, BenchmarkWireError> {
    let raw: RawBenchmarkRunRequest = decode_jsonl(bytes, limits, RecordLimit::Input)?;
    BenchmarkRunRequest::new(raw.identity, raw.resolved_bytes.into_vec())
}
