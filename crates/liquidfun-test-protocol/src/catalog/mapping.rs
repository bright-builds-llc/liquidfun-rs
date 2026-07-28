use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;

use super::{
    CATALOG_MAXIMUM_CANONICAL_BYTES, CATALOG_MAXIMUM_DEFINITIONS, CatalogDefinition, CatalogError,
    CatalogErrorKind, CatalogEvidence, CatalogSlug, ScenarioConsumer, ScenarioEligibility,
    ScenarioVersion,
};

const MAXIMUM_MAPPING_REFERENCES: usize = 32;
const MAXIMUM_REFERENCE_ID_BYTES: usize = 192;
const MAXIMUM_AUTHORITY_BYTES: usize = 4 * 1024 * 1024;
const MAXIMUM_AUTHORITY_RECORDS: usize = 4_096;
const UPSTREAM_CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../reference/upstream-corpus.json"
));
const COMPATIBILITY_LEDGER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../reference/compatibility.json"
));

mod authority;
mod projection;

/// A bounded stable identity resolved against a checked repository authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CatalogReferenceId(Box<str>);

impl CatalogReferenceId {
    /// Parses a bounded repository authority identity.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] for empty, oversized, non-ASCII, or unreviewed characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CatalogError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAXIMUM_REFERENCE_ID_BYTES
            || !value.is_ascii()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_' | b'/')
            })
            || value.contains("//")
            || value.starts_with('/')
            || value.ends_with('/')
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the validated authority identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exactly one evidence authority disposition for a scenario mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogEvidenceDisposition {
    /// Direct oracle artifacts provide the evidence.
    Oracle {
        /// Checked artifact identities from the reference manifest.
        artifacts: Vec<CatalogReferenceId>,
    },
    /// Sealed earlier-phase witnesses provide reviewed equivalent evidence.
    ReviewedEquivalent {
        /// Exact typed evidence leaves from the scenario definition.
        evidence: Vec<CatalogEvidence>,
    },
}

impl CatalogEvidenceDisposition {
    fn is_valid(&self) -> bool {
        match self {
            Self::Oracle { artifacts } => {
                !artifacts.is_empty()
                    && artifacts.len() <= MAXIMUM_MAPPING_REFERENCES
                    && all_unique(artifacts)
            }
            Self::ReviewedEquivalent { evidence } => {
                !evidence.is_empty()
                    && evidence.len() <= MAXIMUM_MAPPING_REFERENCES
                    && all_unique(evidence)
            }
        }
    }
}

/// Validated cross-consumer mapping for one stable scenario identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMapping {
    slug: CatalogSlug,
    scenario_version: ScenarioVersion,
    test_ids: Box<[CatalogSlug]>,
    evidence_disposition: CatalogEvidenceDisposition,
    regression_use: bool,
    benchmark_eligible: bool,
    visual_eligible: bool,
    upstream_corpus_ids: Box<[CatalogReferenceId]>,
    compatibility_refs: Box<[CatalogReferenceId]>,
}

impl CatalogMapping {
    /// Constructs one locally bounded mapping before cross-authority validation.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] for empty, duplicate, or oversized mapping fields.
    #[allow(
        clippy::too_many_arguments,
        reason = "the closed mapping carries each consumer disposition explicitly"
    )]
    pub fn new(
        slug: CatalogSlug,
        scenario_version: ScenarioVersion,
        test_ids: Vec<CatalogSlug>,
        evidence_disposition: CatalogEvidenceDisposition,
        regression_use: bool,
        benchmark_eligible: bool,
        visual_eligible: bool,
        upstream_corpus_ids: Vec<CatalogReferenceId>,
        compatibility_refs: Vec<CatalogReferenceId>,
    ) -> Result<Self, CatalogError> {
        if !valid_references(&test_ids)
            || !evidence_disposition.is_valid()
            || !valid_references(&upstream_corpus_ids)
            || !valid_references(&compatibility_refs)
        {
            return Err(CatalogError::new(CatalogErrorKind::InvalidMetadata));
        }
        Ok(Self {
            slug,
            scenario_version,
            test_ids: test_ids.into_boxed_slice(),
            evidence_disposition,
            regression_use,
            benchmark_eligible,
            visual_eligible,
            upstream_corpus_ids: upstream_corpus_ids.into_boxed_slice(),
            compatibility_refs: compatibility_refs.into_boxed_slice(),
        })
    }

    /// Returns the stable scenario slug.
    #[must_use]
    pub const fn slug(&self) -> &CatalogSlug {
        &self.slug
    }

    /// Returns the stable scenario version.
    #[must_use]
    pub const fn scenario_version(&self) -> ScenarioVersion {
        self.scenario_version
    }

    /// Returns reviewed public test identities.
    #[must_use]
    pub fn test_ids(&self) -> &[CatalogSlug] {
        &self.test_ids
    }

    /// Returns the single evidence authority disposition.
    #[must_use]
    pub const fn evidence_disposition(&self) -> &CatalogEvidenceDisposition {
        &self.evidence_disposition
    }

    /// Returns whether this scenario is required in deterministic regressions.
    #[must_use]
    pub const fn regression_use(&self) -> bool {
        self.regression_use
    }

    /// Returns whether the selected downstream consumer is eligible.
    #[must_use]
    pub const fn is_eligible(&self, consumer: ScenarioConsumer) -> bool {
        match consumer {
            ScenarioConsumer::Regression => self.regression_use,
            ScenarioConsumer::Benchmark => self.benchmark_eligible,
            ScenarioConsumer::Visual => self.visual_eligible,
        }
    }

    /// Returns checked semantic upstream corpus identities.
    #[must_use]
    pub fn upstream_corpus_ids(&self) -> &[CatalogReferenceId] {
        &self.upstream_corpus_ids
    }

    /// Returns checked compatibility-ledger references.
    #[must_use]
    pub fn compatibility_refs(&self) -> &[CatalogReferenceId] {
        &self.compatibility_refs
    }
}

/// One immutable closed registry and its complete validated consumer mappings.
#[derive(Debug)]
pub struct ScenarioCatalog {
    definitions: Box<[CatalogDefinition]>,
    mappings: Box<[CatalogMapping]>,
}

impl ScenarioCatalog {
    /// Validates, sorts, and seals definitions and mappings.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] for any duplicate, missing, unknown, stale, or contradictory row.
    pub fn new(
        mut definitions: Vec<CatalogDefinition>,
        mut mappings: Vec<CatalogMapping>,
    ) -> Result<Self, CatalogError> {
        if definitions.is_empty()
            || definitions.len() > CATALOG_MAXIMUM_DEFINITIONS
            || mappings.len() > CATALOG_MAXIMUM_DEFINITIONS
        {
            return Err(CatalogError::new(CatalogErrorKind::TooManyDefinitions));
        }
        definitions.sort_unstable_by(definition_order);
        mappings.sort_unstable_by(mapping_order);
        reject_duplicate_definitions(&definitions)?;
        reject_duplicate_mappings(&mappings)?;

        let upstream_ids = authority_ids(UPSTREAM_CORPUS, "items")?;
        let compatibility_ids = authority_ids(COMPATIBILITY_LEDGER, "entries")?;
        for mapping in &mappings {
            let Some(definition) = definitions.iter().find(|definition| {
                definition.slug() == mapping.slug()
                    && definition.scenario_version() == mapping.scenario_version()
            }) else {
                return Err(CatalogError::new(CatalogErrorKind::UnknownMapping));
            };
            validate_mapping(definition, mapping, &upstream_ids, &compatibility_ids)?;
        }
        if definitions.iter().any(|definition| {
            !mappings.iter().any(|mapping| {
                definition.slug() == mapping.slug()
                    && definition.scenario_version() == mapping.scenario_version()
            })
        }) {
            return Err(CatalogError::new(CatalogErrorKind::MissingMapping));
        }
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            mappings: mappings.into_boxed_slice(),
        })
    }

    /// Returns definitions in stable slug/version order.
    #[must_use]
    pub fn definitions(&self) -> &[CatalogDefinition] {
        &self.definitions
    }

    /// Returns mappings in the same stable identity order.
    #[must_use]
    pub fn mappings(&self) -> &[CatalogMapping] {
        &self.mappings
    }

    /// Finds the mapping for one typed stable identity.
    #[must_use]
    pub fn mapping(
        &self,
        slug: &CatalogSlug,
        scenario_version: ScenarioVersion,
    ) -> Option<&CatalogMapping> {
        self.mappings.iter().find(|mapping| {
            mapping.slug() == slug && mapping.scenario_version() == scenario_version
        })
    }

    /// Parses text strictly as a stable slug, never as a display title.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] for title misuse, malformed identity, or an unknown mapping.
    pub fn mapping_by_text(
        &self,
        identity: &str,
        scenario_version: ScenarioVersion,
    ) -> Result<&CatalogMapping, CatalogError> {
        if self
            .definitions
            .iter()
            .any(|definition| definition.display_title() == identity)
        {
            return Err(CatalogError::new(CatalogErrorKind::TitleAsIdentity));
        }
        let slug = CatalogSlug::new(identity.to_owned())?;
        self.mapping(&slug, scenario_version)
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::UnknownMapping))
    }
}

/// Returns every reviewed scenario family in one stable order.
///
/// # Errors
///
/// Returns [`CatalogError`] if a family definition violates its typed invariants.
pub fn scenario_definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    super::scenarios::scenario_definitions()
}

/// Derives complete mappings from definition metadata and reviewed authority joins.
///
/// # Errors
///
/// Returns [`CatalogError`] if metadata or a reviewed join row is incomplete.
pub fn scenario_mappings(
    definitions: &[CatalogDefinition],
) -> Result<Vec<CatalogMapping>, CatalogError> {
    definitions.iter().map(mapping_for_definition).collect()
}

/// Builds the one reviewed runtime catalog authority.
///
/// # Errors
///
/// Returns [`CatalogError`] if any definition, mapping, or authority join drifts.
pub fn reviewed_scenario_catalog() -> Result<ScenarioCatalog, CatalogError> {
    let definitions = scenario_definitions()?;
    let mappings = scenario_mappings(&definitions)?;
    ScenarioCatalog::new(definitions, mappings)
}

/// Renders the deterministic review projection from the typed catalog.
///
/// # Errors
///
/// Returns [`CatalogError`] if the catalog cannot be projected within reviewed bounds.
pub fn render_scenario_catalog_projection() -> Result<Vec<u8>, CatalogError> {
    let catalog = reviewed_scenario_catalog()?;
    let rendered = projection::render(&catalog)?;
    if rendered.len() > CATALOG_MAXIMUM_CANONICAL_BYTES {
        return Err(CatalogError::new(CatalogErrorKind::CanonicalBytesExceeded));
    }
    Ok(rendered)
}

/// Checks tracked projection bytes against the in-memory typed authority without writing.
///
/// # Errors
///
/// Returns [`CatalogError`] when the bytes are oversized, malformed, or differ exactly.
pub fn check_scenario_catalog_projection(tracked: &[u8]) -> Result<(), CatalogError> {
    if tracked.len() > CATALOG_MAXIMUM_CANONICAL_BYTES {
        return Err(CatalogError::new(CatalogErrorKind::CanonicalBytesExceeded));
    }
    let rendered = render_scenario_catalog_projection()?;
    if rendered != tracked {
        return Err(CatalogError::new(CatalogErrorKind::ProjectionMismatch));
    }
    Ok(())
}

fn mapping_for_definition(definition: &CatalogDefinition) -> Result<CatalogMapping, CatalogError> {
    let metadata = definition
        .metadata()
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::MissingMapping))?;
    let (upstream, compatibility) = authority_references(definition.slug().as_str())
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::MissingMapping))?;
    CatalogMapping::new(
        definition.slug().clone(),
        definition.scenario_version(),
        metadata.coverage().test_ids().to_vec(),
        CatalogEvidenceDisposition::ReviewedEquivalent {
            evidence: metadata.coverage().evidence_leaves().to_vec(),
        },
        metadata
            .coverage()
            .is_eligible(ScenarioConsumer::Regression),
        metadata.coverage().is_eligible(ScenarioConsumer::Benchmark),
        metadata.coverage().is_eligible(ScenarioConsumer::Visual),
        reference_ids(upstream)?,
        reference_ids(compatibility)?,
    )
}

fn validate_mapping(
    definition: &CatalogDefinition,
    mapping: &CatalogMapping,
    upstream_ids: &BTreeSet<String>,
    compatibility_ids: &BTreeSet<String>,
) -> Result<(), CatalogError> {
    let metadata = definition
        .metadata()
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::MissingMapping))?;
    if mapping
        .test_ids()
        .iter()
        .any(|id| !known_test_id(id.as_str()))
    {
        return Err(CatalogError::new(CatalogErrorKind::StaleTestId));
    }
    match mapping.evidence_disposition() {
        CatalogEvidenceDisposition::ReviewedEquivalent { evidence }
            if evidence == metadata.coverage().evidence_leaves() => {}
        CatalogEvidenceDisposition::ReviewedEquivalent { .. } => {
            return Err(CatalogError::new(CatalogErrorKind::StaleEvidence));
        }
        CatalogEvidenceDisposition::Oracle { .. } => {
            return Err(CatalogError::new(
                CatalogErrorKind::ContradictoryEligibility,
            ));
        }
    }
    for consumer in ScenarioConsumer::ALL {
        if mapping.is_eligible(consumer) != metadata.coverage().is_eligible(consumer) {
            return Err(CatalogError::new(
                CatalogErrorKind::ContradictoryEligibility,
            ));
        }
    }
    if mapping
        .upstream_corpus_ids()
        .iter()
        .any(|id| !upstream_ids.contains(id.as_str()))
    {
        return Err(CatalogError::new(CatalogErrorKind::StaleUpstreamCorpusId));
    }
    if mapping
        .compatibility_refs()
        .iter()
        .any(|id| !compatibility_ids.contains(id.as_str()))
    {
        return Err(CatalogError::new(CatalogErrorKind::StaleCompatibilityRef));
    }
    if definition.eligibility() == ScenarioEligibility::SeedRequired
        && (definition.generator_id().as_str().is_empty()
            || definition.generator_version() != super::GeneratorVersion::CURRENT)
    {
        return Err(CatalogError::new(CatalogErrorKind::SeedGeneratorMissing));
    }
    Ok(())
}

fn reject_duplicate_definitions(definitions: &[CatalogDefinition]) -> Result<(), CatalogError> {
    if definitions.windows(2).any(|pair| {
        pair[0].slug() == pair[1].slug() && pair[0].scenario_version() == pair[1].scenario_version()
    }) {
        return Err(CatalogError::new(
            CatalogErrorKind::DuplicateScenarioIdentity,
        ));
    }
    Ok(())
}

fn reject_duplicate_mappings(mappings: &[CatalogMapping]) -> Result<(), CatalogError> {
    if mappings.windows(2).any(|pair| {
        pair[0].slug() == pair[1].slug() && pair[0].scenario_version() == pair[1].scenario_version()
    }) {
        return Err(CatalogError::new(CatalogErrorKind::DuplicateMapping));
    }
    Ok(())
}

fn definition_order(left: &CatalogDefinition, right: &CatalogDefinition) -> std::cmp::Ordering {
    (left.slug(), left.scenario_version()).cmp(&(right.slug(), right.scenario_version()))
}

fn mapping_order(left: &CatalogMapping, right: &CatalogMapping) -> std::cmp::Ordering {
    (left.slug(), left.scenario_version()).cmp(&(right.slug(), right.scenario_version()))
}

fn authority_ids(input: &str, array_field: &str) -> Result<BTreeSet<String>, CatalogError> {
    if input.len() > MAXIMUM_AUTHORITY_BYTES {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    let parsed: Value = serde_json::from_str(input)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    let records = parsed
        .get(array_field)
        .and_then(Value::as_array)
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    if records.len() > MAXIMUM_AUTHORITY_RECORDS {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    let mut ids = BTreeSet::new();
    for record in records {
        let id = record
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
        let validated = CatalogReferenceId::new(id.to_owned())?;
        if !ids.insert(validated.as_str().to_owned()) {
            return Err(CatalogError::new(CatalogErrorKind::InvalidMetadata));
        }
    }
    Ok(ids)
}

fn valid_references<T: Eq>(references: &[T]) -> bool {
    !references.is_empty()
        && references.len() <= MAXIMUM_MAPPING_REFERENCES
        && all_unique(references)
}

fn all_unique<T: Eq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .all(|(index, value)| !values[index + 1..].contains(value))
}

fn reference_ids(values: &[&str]) -> Result<Vec<CatalogReferenceId>, CatalogError> {
    values
        .iter()
        .map(|value| CatalogReferenceId::new(*value))
        .collect()
}

fn known_test_id(id: &str) -> bool {
    matches!(
        id,
        "rigid-world-lifecycle-test"
            | "rigid-world-contact-test"
            | "rigid-world-stack-test"
            | "rigid-world-sleep-test"
            | "rigid-world-continuous-test"
            | "rigid-world-filter-test"
            | "rigid-world-query-test"
            | "rigid-world-callback-test"
            | "rigid-world-mutation-test"
            | "rigid-world-destruction-test"
            | "joint-revolute-test"
            | "joint-prismatic-test"
            | "joint-distance-test"
            | "joint-pulley-test"
            | "joint-mouse-test"
            | "joint-gear-test"
            | "joint-wheel-test"
            | "joint-weld-test"
            | "joint-friction-test"
            | "joint-rope-test"
            | "joint-motor-test"
            | "standalone-rope-test"
            | "particle-lifecycle"
            | "particle-body-contacts"
            | "particle-forces-statistics"
            | "particle-solver-flags"
            | "particle-group-mutation"
            | "particle-queries"
    )
}

fn authority_references(slug: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    authority::references(slug)
}
