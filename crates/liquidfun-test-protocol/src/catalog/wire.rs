//! Exact resolved-byte run request for cross-process catalog execution.

use serde::{Deserialize, Serialize};

use super::{
    CATALOG_MAXIMUM_CANONICAL_BYTES, CatalogErrorKind, CatalogSchemaVersion, CatalogSlug,
    GeneratorId, GeneratorVersion, ResolvedScenario, RunSettings, ScenarioVersion,
    decode_resolved_scenario,
};
use crate::{
    CheckpointDecodeError, CheckpointErrorKind, CheckpointValidationError, CodecError,
    EvidenceTier, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, Sha256Hex,
    checkpoint::validation, codec::BoundedVec, decode_jsonl, encode_jsonl,
};

/// Pinned identities required before one resolved request may produce evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunProvenanceRequirements {
    required_identity_sha256: Sha256Hex,
    limits_profile_sha256: Sha256Hex,
    evidence_tier: EvidenceTier,
}

impl RunProvenanceRequirements {
    /// Creates one closed provenance requirement set.
    #[must_use]
    pub const fn new(
        required_identity_sha256: Sha256Hex,
        limits_profile_sha256: Sha256Hex,
        evidence_tier: EvidenceTier,
    ) -> Self {
        Self {
            required_identity_sha256,
            limits_profile_sha256,
            evidence_tier,
        }
    }

    /// Returns the minimum evidence tier required by this request.
    #[must_use]
    pub const fn evidence_tier(&self) -> EvidenceTier {
        self.evidence_tier
    }

    /// Returns the exact reviewed child build identity required by this run.
    #[must_use]
    pub const fn required_identity_sha256(&self) -> &Sha256Hex {
        &self.required_identity_sha256
    }

    /// Returns the exact immutable resource-profile identity required by this run.
    #[must_use]
    pub const fn limits_profile_sha256(&self) -> &Sha256Hex {
        &self.limits_profile_sha256
    }
}

/// Strict request carrying the exact canonical resolved scenario bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRunRequest {
    request_id: RequestId,
    resolved: ResolvedScenario,
    provenance_requirements: RunProvenanceRequirements,
}

impl CatalogRunRequest {
    /// Creates a request after replay-validating its exact resolved bytes and hash.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointValidationError`] if the resolved bytes are contradictory.
    pub fn new(
        request_id: RequestId,
        resolved: ResolvedScenario,
        provenance_requirements: RunProvenanceRequirements,
    ) -> Result<Self, CheckpointValidationError> {
        decode_resolved_scenario(
            resolved.canonical_bytes(),
            resolved.identity().content_sha256(),
        )
        .map_err(|error| map_catalog_validation_error(&error))?;
        Ok(Self {
            request_id,
            resolved,
            provenance_requirements,
        })
    }

    /// Returns the exact decoded resolved scenario.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedScenario {
        &self.resolved
    }

    /// Returns the stable request identity shared by both engine captures.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns the pinned provenance requirements.
    #[must_use]
    pub const fn provenance_requirements(&self) -> &RunProvenanceRequirements {
        &self.provenance_requirements
    }
}

#[derive(Serialize)]
struct CatalogRunRequestRef<'a> {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    request_id: &'a RequestId,
    catalog_schema_version: CatalogSchemaVersion,
    slug: &'a CatalogSlug,
    scenario_version: ScenarioVersion,
    generator_id: &'a GeneratorId,
    generator_version: GeneratorVersion,
    maybe_seed: Option<u64>,
    settings: RunSettings,
    resolved_bytes: &'a [u8],
    resolved_sha256: &'a Sha256Hex,
    provenance_requirements: &'a RunProvenanceRequirements,
}

impl Serialize for CatalogRunRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let identity = self.resolved.identity();
        CatalogRunRequestRef {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: "catalog_run_request",
            request_id: &self.request_id,
            catalog_schema_version: identity.catalog_schema_version(),
            slug: identity.slug(),
            scenario_version: identity.scenario_version(),
            generator_id: identity.generator_id(),
            generator_version: identity.generator_version(),
            maybe_seed: identity.maybe_seed(),
            settings: identity.settings(),
            resolved_bytes: self.resolved.canonical_bytes(),
            resolved_sha256: identity.content_sha256(),
            provenance_requirements: &self.provenance_requirements,
        }
        .serialize(serializer)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalogRunRequest {
    #[serde(rename = "protocol_version")]
    _protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: CatalogRunRequestRecordKind,
    request_id: RequestId,
    catalog_schema_version: CatalogSchemaVersion,
    slug: CatalogSlug,
    scenario_version: ScenarioVersion,
    generator_id: GeneratorId,
    generator_version: GeneratorVersion,
    maybe_seed: Option<u64>,
    settings: RunSettings,
    resolved_bytes: BoundedVec<u8, CATALOG_MAXIMUM_CANONICAL_BYTES>,
    resolved_sha256: Sha256Hex,
    provenance_requirements: RunProvenanceRequirements,
}

#[derive(Deserialize)]
enum CatalogRunRequestRecordKind {
    #[serde(rename = "catalog_run_request")]
    CatalogRunRequest,
}

/// Encodes one exact resolved-byte request as strict newline-complete JSON.
///
/// # Errors
///
/// Returns [`CodecError`] when serialization or the input byte bound fails.
pub fn encode_catalog_run_request_jsonl(
    request: &CatalogRunRequest,
    limits: &HarnessLimits,
) -> Result<Vec<u8>, CodecError> {
    encode_jsonl(request, limits, RecordLimit::Input)
}

/// Strictly decodes one exact resolved-byte request without rerunning its generator.
///
/// # Errors
///
/// Returns [`CheckpointDecodeError`] for framing, bounds, hash, or identity contradictions.
pub fn decode_catalog_run_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<CatalogRunRequest, CheckpointDecodeError> {
    let raw: RawCatalogRunRequest = decode_jsonl(bytes, limits, RecordLimit::Input)?;
    let resolved_bytes = raw.resolved_bytes.into_vec();
    let resolved = decode_resolved_scenario(&resolved_bytes, &raw.resolved_sha256)
        .map_err(|error| map_catalog_validation_error(&error))?;
    let identity = resolved.identity();
    if identity.catalog_schema_version() != raw.catalog_schema_version
        || identity.slug() != &raw.slug
        || identity.scenario_version() != raw.scenario_version
        || identity.generator_id() != &raw.generator_id
        || identity.generator_version() != raw.generator_version
        || identity.maybe_seed() != raw.maybe_seed
        || identity.settings() != raw.settings
    {
        return Err(validation(CheckpointErrorKind::IdentityMismatch).into());
    }
    Ok(CatalogRunRequest {
        request_id: raw.request_id,
        resolved,
        provenance_requirements: raw.provenance_requirements,
    })
}

fn map_catalog_validation_error(error: &super::CatalogError) -> CheckpointValidationError {
    let kind = if error.kind() == CatalogErrorKind::HashMismatch {
        CheckpointErrorKind::HashMismatch
    } else {
        CheckpointErrorKind::IdentityMismatch
    };
    validation(kind)
}
