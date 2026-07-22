mod corpus;
#[path = "inventory/corpus/discovery.rs"]
mod corpus_discovery;
mod discovery;
mod report;
mod validation;

// Keep the checked parser in the production inventory graph before a later
// plan adds the first authoritative corpus manifest.
const _: fn(&[u8], &str) -> Result<corpus::CorpusManifest, corpus::CorpusError> =
    corpus::parse_manifest;

use std::env;
use std::error::Error;
use std::fmt::{self, Arguments, Display, Formatter, Write as _};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

const USAGE: &str = r"Usage: cargo xtask inventory <command>

Commands:
  discover   Explicitly refresh reference/discovery.json from the pinned tree
  generate   Explicitly refresh COMPATIBILITY.md from validated ledgers
  check      Read-only validation of schemas, coverage, discovery, and report
  check-report
             Read-only validation of schemas, coverage, and generated report
  corpus refresh
             Refresh reference/upstream-corpus.json from the verified pinned tree
  corpus check-snapshot
             Validate canonical corpus snapshot bytes without reading third_party";
const SCHEMA_VERSION: u64 = 1;
const EVIDENCE_DIMENSIONS: [&str; 8] = [
    "investigated",
    "planned",
    "implemented",
    "unit_tested",
    "differentially_validated",
    "platform_validated",
    "documented_difference",
    "intentionally_unsupported",
];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InventoryError {
    category: &'static str,
    message: String,
}

impl InventoryError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for InventoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "inventory/{}: {}", self.category, self.message)
    }
}

impl Error for InventoryError {}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum CompatibilityKind {
    Subsystem,
    PublicApi,
    SourceArea,
    Test,
    Example,
    BuildOption,
}

impl CompatibilityKind {
    const ALL: [Self; 6] = [
        Self::Subsystem,
        Self::PublicApi,
        Self::SourceArea,
        Self::Test,
        Self::Example,
        Self::BuildOption,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Subsystem => "subsystem",
            Self::PublicApi => "public_api",
            Self::SourceArea => "source_area",
            Self::Test => "test",
            Self::Example => "example",
            Self::BuildOption => "build_option",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum DiscoveryKind {
    PublicHeader,
    SourceArea,
    Test,
    Example,
    BuildOption,
}

impl DiscoveryKind {
    const fn rank(self) -> u8 {
        match self {
            Self::PublicHeader => 0,
            Self::SourceArea => 1,
            Self::Test => 2,
            Self::Example => 3,
            Self::BuildOption => 4,
        }
    }

    const fn compatibility_kind(self) -> CompatibilityKind {
        match self {
            Self::PublicHeader => CompatibilityKind::PublicApi,
            Self::SourceArea => CompatibilityKind::SourceArea,
            Self::Test => CompatibilityKind::Test,
            Self::Example => CompatibilityKind::Example,
            Self::BuildOption => CompatibilityKind::BuildOption,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityLedger {
    schema_version: u64,
    oracle_revision: String,
    sort_contract: String,
    evidence_dimensions: Vec<String>,
    entries: Vec<CompatibilityEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityEntry {
    id: String,
    kind: CompatibilityKind,
    upstream_path: String,
    upstream_symbol: Option<String>,
    applicability: Applicability,
    rust_target: String,
    provenance_ref: String,
    notice_refs: Vec<String>,
    evidence: Evidence,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Applicability {
    status: ApplicabilityStatus,
    rationale: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ApplicabilityStatus {
    Applicable,
    ReviewedExclusion,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    investigated: EvidenceRecord,
    planned: EvidenceRecord,
    implemented: EvidenceRecord,
    unit_tested: EvidenceRecord,
    differentially_validated: EvidenceRecord,
    platform_validated: EvidenceRecord,
    documented_difference: EvidenceRecord,
    intentionally_unsupported: EvidenceRecord,
}

impl Evidence {
    fn records(&self) -> [(&'static str, &EvidenceRecord); 8] {
        [
            ("investigated", &self.investigated),
            ("planned", &self.planned),
            ("implemented", &self.implemented),
            ("unit_tested", &self.unit_tested),
            ("differentially_validated", &self.differentially_validated),
            ("platform_validated", &self.platform_validated),
            ("documented_difference", &self.documented_difference),
            ("intentionally_unsupported", &self.intentionally_unsupported),
        ]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceRecord {
    status: EvidenceStatus,
    references: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EvidenceStatus {
    Evidenced,
    NotEvidenced,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DiscoveryLedger {
    schema_version: u64,
    oracle_revision: String,
    sort_contract: String,
    scopes: Vec<DiscoveryScope>,
    entries: Vec<DiscoveryEntry>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct DiscoveryScope {
    kind: DiscoveryKind,
    root: String,
    matcher: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct DiscoveryEntry {
    kind: DiscoveryKind,
    upstream_path: String,
    upstream_symbol: Option<String>,
}

pub(crate) fn run(args: &[String]) -> Result<(), InventoryError> {
    let repository_root = repository_root()?;
    let oracle_revision = read_oracle_revision(&repository_root)?;

    match args {
        [namespace, command] if namespace == "corpus" && command == "refresh" => {
            let count = corpus_discovery::refresh(&repository_root, &oracle_revision)?;
            println!("semantic corpus refreshed: {count} items");
            Ok(())
        }
        [namespace, command] if namespace == "corpus" && command == "check-snapshot" => {
            let count = corpus_discovery::check_snapshot(&repository_root, &oracle_revision)?;
            println!("semantic corpus snapshot verified: {count} items");
            Ok(())
        }
        [command] => match command.as_str() {
            "discover" => discover(&repository_root, &oracle_revision),
            "generate" => generate(&repository_root, &oracle_revision),
            "check" => check(&repository_root, &oracle_revision),
            "check-report" => check_report(&repository_root, &oracle_revision),
            unknown => Err(InventoryError::usage(format!(
                "unknown inventory command `{unknown}`"
            ))),
        },
        _ => Err(InventoryError::usage(
            "expected a closed inventory command shape",
        )),
    }
}

fn discover(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let snapshot = discovery::scan(repository_root, oracle_revision)?;
    let contents = format_discovery(&snapshot)?;
    let path = repository_root.join("reference/discovery.json");
    fs::write(&path, contents).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })?;
    println!(
        "inventory discovery refreshed: {} entries",
        snapshot.entries.len()
    );
    Ok(())
}

fn generate(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_discovery(repository_root, oracle_revision)?;
    let contents = report::render(&compatibility);
    let path = repository_root.join("COMPATIBILITY.md");
    fs::write(&path, contents).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })?;
    println!(
        "compatibility report generated: {} entries",
        compatibility.entries.len()
    );
    Ok(())
}

fn check(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_discovery(repository_root, oracle_revision)?;
    require_current_report(repository_root, &compatibility)?;
    println!(
        "inventory verified: {} compatibility rows",
        compatibility.entries.len()
    );
    Ok(())
}

fn check_report(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_report(repository_root, &compatibility)?;
    println!(
        "compatibility report verified: {} rows",
        compatibility.entries.len()
    );
    Ok(())
}

fn require_current_report(
    repository_root: &Path,
    compatibility: &CompatibilityLedger,
) -> Result<(), InventoryError> {
    let expected_report = report::render(compatibility);
    require_exact_file(
        &repository_root.join("COMPATIBILITY.md"),
        &expected_report,
        "report",
        "run `cargo xtask inventory generate`",
    )
}

fn validated_ledgers(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(CompatibilityLedger, DiscoveryLedger), InventoryError> {
    let compatibility: CompatibilityLedger = read_json(
        &repository_root.join("reference/compatibility.json"),
        "compatibility schema",
    )?;
    let discovery: DiscoveryLedger = read_json(
        &repository_root.join("reference/discovery.json"),
        "discovery schema",
    )?;
    validation::compatibility(&compatibility, oracle_revision, repository_root)?;
    validation::discovery(&discovery, oracle_revision)?;
    validation::coverage(&compatibility, &discovery)?;
    Ok((compatibility, discovery))
}

fn require_current_discovery(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    let expected = format_discovery(&discovery::scan(repository_root, oracle_revision)?)?;
    require_exact_file(
        &repository_root.join("reference/discovery.json"),
        &expected,
        "discovery",
        "run `cargo xtask inventory discover` and review the compatibility ledger",
    )
}

fn require_exact_file(
    path: &Path,
    expected: &str,
    label: &str,
    remedy: &str,
) -> Result<(), InventoryError> {
    let actual = fs::read_to_string(path).map_err(|error| {
        InventoryError::new(
            "stale",
            format!("failed to read {}: {error}; {remedy}", path.display()),
        )
    })?;
    if actual == expected {
        return Ok(());
    }
    Err(InventoryError::new(
        "stale",
        format!("{label} does not match deterministic generated bytes; {remedy}"),
    ))
}

fn format_discovery(ledger: &DiscoveryLedger) -> Result<String, InventoryError> {
    let mut output = String::new();
    output.push_str("{\n");
    append_formatted(
        &mut output,
        format_args!("  \"schema_version\": {},\n", ledger.schema_version),
    );
    append_formatted(
        &mut output,
        format_args!(
            "  \"oracle_revision\": {},\n",
            json_string(&ledger.oracle_revision)?
        ),
    );
    append_formatted(
        &mut output,
        format_args!(
            "  \"sort_contract\": {},\n",
            json_string(&ledger.sort_contract)?
        ),
    );
    output.push_str("  \"scopes\": [\n");
    append_json_rows(&mut output, &ledger.scopes)?;
    output.push_str("  ],\n  \"entries\": [\n");
    append_json_rows(&mut output, &ledger.entries)?;
    output.push_str("  ]\n}\n");
    Ok(output)
}

fn append_formatted(output: &mut String, arguments: Arguments<'_>) {
    output
        .write_fmt(arguments)
        .expect("writing formatted discovery data to a String cannot fail");
}

fn append_json_rows<T: Serialize>(output: &mut String, rows: &[T]) -> Result<(), InventoryError> {
    for (index, row) in rows.iter().enumerate() {
        let serialized = serde_json::to_string(row).map_err(|error| {
            InventoryError::new("schema", format!("failed to serialize discovery: {error}"))
        })?;
        output.push_str("    ");
        output.push_str(&serialized);
        if index + 1 != rows.len() {
            output.push(',');
        }
        output.push('\n');
    }
    Ok(())
}

fn json_string(value: &str) -> Result<String, InventoryError> {
    serde_json::to_string(value).map_err(|error| {
        InventoryError::new(
            "schema",
            format!("failed to serialize JSON string: {error}"),
        )
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, InventoryError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        InventoryError::new(
            "schema",
            format!("invalid {label} in {}: {error}", path.display()),
        )
    })
}

fn read_oracle_revision(repository_root: &Path) -> Result<String, InventoryError> {
    let path = repository_root.join("reference/upstream-lock.toml");
    let contents = fs::read_to_string(&path).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value: toml::Value = toml::from_str(&contents).map_err(|error| {
        InventoryError::new("schema", format!("invalid {}: {error}", path.display()))
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(toml::Value::as_integer);
    if schema_version != Some(1) {
        return Err(InventoryError::new(
            "schema",
            "upstream lock schema_version must be 1",
        ));
    }
    let Some(revision) = value.get("revision").and_then(toml::Value::as_str) else {
        return Err(InventoryError::new(
            "schema",
            "upstream lock is missing revision",
        ));
    };
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(InventoryError::new(
            "revision",
            "upstream revision must be lowercase 40-hex",
        ));
    }
    Ok(revision.to_owned())
}

fn require_schema_and_revision(
    schema_version: u64,
    actual_revision: &str,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    if schema_version != SCHEMA_VERSION {
        return Err(InventoryError::new(
            "schema",
            format!("expected schema version {SCHEMA_VERSION}, actual {schema_version}"),
        ));
    }
    if actual_revision != oracle_revision {
        return Err(InventoryError::new(
            "revision",
            format!(
                "oracle revision mismatch: expected `{oracle_revision}`, actual `{actual_revision}`"
            ),
        ));
    }
    Ok(())
}

fn validate_relative_path(value: &str, field: &str) -> Result<(), InventoryError> {
    if value.is_empty() || value.contains('\\') {
        return Err(InventoryError::new(
            "path",
            format!("{field} must be a normalized relative UTF-8 path"),
        ));
    }
    let path = Path::new(value);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(InventoryError::new(
            "path",
            format!("{field} `{value}` must not be absolute or contain traversal"),
        ));
    }
    Ok(())
}

fn repository_root() -> Result<PathBuf, InventoryError> {
    if let Some(root) = env::var_os("LIQUIDFUN_XTASK_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let current_dir = env::current_dir().map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("reference/upstream-lock.toml").is_file()
            && candidate.join("Cargo.toml").is_file()
    }) else {
        return Err(InventoryError::new(
            "repository",
            "could not find Cargo.toml and reference/upstream-lock.toml",
        ));
    };
    Ok(root.to_path_buf())
}
