//! Scene fingerprint gate: joins scene-spot fingerprints with the pinned baseline by scene id.
//!
//! Every protected baseline id must be present with an identical fingerprint.
//! Ids the baseline does not know are accepted and listed as additions, so the
//! catalog can grow without editing the baseline in the same change.

use std::collections::{BTreeMap, BTreeSet};

use super::PlaygroundError;
use super::spot;

#[cfg(test)]
mod tests;

const ERROR_KIND: &str = "gate";
const BASELINE_KIND: &str = "scene_fingerprint_baseline";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const OBSERVED_KIND: &str = "native_scene_spot";

/// One scene id with its 16-digit lowercase hex end-state fingerprint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SceneFingerprint {
    pub(super) id: String,
    pub(super) fingerprint: String,
}

/// A protected scene whose observed fingerprint differs from the baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChangedScene {
    pub(super) id: String,
    pub(super) baseline: String,
    pub(super) observed: String,
}

/// Result of joining protected and observed fingerprints by scene id.
///
/// `unchanged`, `changed` and `missing` follow baseline order; `added` follows
/// observed order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct GateReport {
    pub(super) unchanged: Vec<String>,
    pub(super) changed: Vec<ChangedScene>,
    pub(super) missing: Vec<String>,
    pub(super) added: Vec<SceneFingerprint>,
}

impl GateReport {
    /// The gate passes when no protected id changed or went missing.
    pub(super) fn passes(&self) -> bool {
        self.changed.is_empty() && self.missing.is_empty()
    }

    fn protected_count(&self) -> usize {
        self.unchanged.len() + self.changed.len() + self.missing.len()
    }
}

/// Parses the committed `scene_fingerprint_baseline` (schema 1, `scenes[] { id, fingerprint }`).
///
/// # Errors
///
/// Returns a `gate` error for a wrong kind or schema version, an empty or
/// malformed `scenes` list, an invalid fingerprint, or a duplicate id.
pub(super) fn parse_baseline(
    json: &serde_json::Value,
) -> Result<Vec<SceneFingerprint>, PlaygroundError> {
    require_kind(json, BASELINE_KIND, "baseline")?;
    let schema_version = json
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    if schema_version != Some(BASELINE_SCHEMA_VERSION) {
        return Err(gate_error(format!(
            "baseline requires schema_version {BASELINE_SCHEMA_VERSION}"
        )));
    }
    let scenes = parse_scene_list(json, "id", "baseline")?;
    if scenes.is_empty() {
        return Err(gate_error("baseline protects no scene ids"));
    }
    Ok(scenes)
}

/// Parses a `native_scene_spot` capture (`scenes[] { scene, fingerprint }`).
///
/// # Errors
///
/// Returns a `gate` error for a wrong kind, a malformed `scenes` list, an
/// invalid fingerprint, or a duplicate id.
pub(super) fn parse_observed(
    json: &serde_json::Value,
) -> Result<Vec<SceneFingerprint>, PlaygroundError> {
    require_kind(json, OBSERVED_KIND, "observed")?;
    parse_scene_list(json, "scene", "observed")
}

/// Joins `protected` and `observed` by scene id.
///
/// # Errors
///
/// Returns a `gate` error when `protected` is empty or either slice repeats an id.
pub(super) fn compare_fingerprints(
    protected: &[SceneFingerprint],
    observed: &[SceneFingerprint],
) -> Result<GateReport, PlaygroundError> {
    if protected.is_empty() {
        return Err(gate_error("baseline protects no scene ids"));
    }
    ensure_unique_ids(protected, "baseline")?;
    ensure_unique_ids(observed, "observed")?;
    let observed_by_id: BTreeMap<&str, &str> = observed
        .iter()
        .map(|scene| (scene.id.as_str(), scene.fingerprint.as_str()))
        .collect();
    let mut report = GateReport::default();
    for scene in protected {
        match observed_by_id.get(scene.id.as_str()) {
            None => report.missing.push(scene.id.clone()),
            Some(&fingerprint) if fingerprint == scene.fingerprint => {
                report.unchanged.push(scene.id.clone());
            }
            Some(&fingerprint) => report.changed.push(ChangedScene {
                id: scene.id.clone(),
                baseline: scene.fingerprint.clone(),
                observed: fingerprint.to_owned(),
            }),
        }
    }
    let protected_ids: BTreeSet<&str> = protected.iter().map(|scene| scene.id.as_str()).collect();
    report.added = observed
        .iter()
        .filter(|scene| !protected_ids.contains(scene.id.as_str()))
        .cloned()
        .collect();
    Ok(report)
}

/// Renders the plain-text gate report pasted into phase summaries.
pub(super) fn render_report(
    report: &GateReport,
    baseline_label: &str,
    observed_label: &str,
) -> String {
    let verdict = if report.passes() { "PASS" } else { "FAIL" };
    let changed = list_or_none(
        report
            .changed
            .iter()
            .map(|scene| format!("{} {} -> {}", scene.id, scene.baseline, scene.observed)),
    );
    let missing = list_or_none(report.missing.iter().cloned());
    let added = list_or_none(
        report
            .added
            .iter()
            .map(|scene| format!("{} {}", scene.id, scene.fingerprint)),
    );
    format!(
        "Scene fingerprint gate: {verdict}\n\
         - protected: {}/{} unchanged\n\
         - changed: {changed}\n\
         - missing: {missing}\n\
         - added (accepted): {added}\n\
         - baseline: {baseline_label}\n\
         - observed: {observed_label}",
        report.unchanged.len(),
        report.protected_count(),
    )
}

fn list_or_none(items: impl Iterator<Item = String>) -> String {
    let items: Vec<String> = items.collect();
    if items.is_empty() {
        String::from("none")
    } else {
        items.join(", ")
    }
}

fn require_kind(
    json: &serde_json::Value,
    expected: &str,
    source: &str,
) -> Result<(), PlaygroundError> {
    let maybe_kind = json.get("kind").and_then(serde_json::Value::as_str);
    if maybe_kind == Some(expected) {
        return Ok(());
    }
    Err(gate_error(format!(
        "{source} requires kind `{expected}`, found {}",
        maybe_kind.map_or_else(|| String::from("none"), |kind| format!("`{kind}`"))
    )))
}

fn parse_scene_list(
    json: &serde_json::Value,
    id_field: &str,
    source: &str,
) -> Result<Vec<SceneFingerprint>, PlaygroundError> {
    let Some(entries) = json.get("scenes").and_then(serde_json::Value::as_array) else {
        return Err(gate_error(format!("{source} requires a `scenes` array")));
    };
    let scenes = entries
        .iter()
        .map(|entry| parse_scene_entry(entry, id_field, source))
        .collect::<Result<Vec<_>, _>>()?;
    ensure_unique_ids(&scenes, source)?;
    Ok(scenes)
}

fn parse_scene_entry(
    entry: &serde_json::Value,
    id_field: &str,
    source: &str,
) -> Result<SceneFingerprint, PlaygroundError> {
    let Some(id) = entry
        .get(id_field)
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
    else {
        return Err(gate_error(format!(
            "{source} scenes require a non-empty `{id_field}` string"
        )));
    };
    let Some(fingerprint) = entry
        .get("fingerprint")
        .and_then(serde_json::Value::as_str)
        .filter(|fingerprint| spot::is_fingerprint(fingerprint))
    else {
        return Err(gate_error(format!(
            "{source} scene `{id}` requires a 16-digit lowercase hex fingerprint"
        )));
    };
    Ok(SceneFingerprint {
        id: id.to_owned(),
        fingerprint: fingerprint.to_owned(),
    })
}

fn ensure_unique_ids(scenes: &[SceneFingerprint], source: &str) -> Result<(), PlaygroundError> {
    let mut seen = BTreeSet::new();
    for scene in scenes {
        if !seen.insert(scene.id.as_str()) {
            return Err(gate_error(format!(
                "{source} repeats scene id `{}`",
                scene.id
            )));
        }
    }
    Ok(())
}

fn gate_error(message: impl Into<String>) -> PlaygroundError {
    PlaygroundError::new(ERROR_KIND, message)
}
