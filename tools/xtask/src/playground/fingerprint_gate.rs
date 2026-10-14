//! Scene fingerprint gate: joins scene-spot fingerprints with the pinned baseline by scene id.

use super::PlaygroundError;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SceneFingerprint {
    pub(super) id: String,
    pub(super) fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ChangedScene {
    pub(super) id: String,
    pub(super) baseline: String,
    pub(super) observed: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct GateReport {
    pub(super) unchanged: Vec<String>,
    pub(super) changed: Vec<ChangedScene>,
    pub(super) missing: Vec<String>,
    pub(super) added: Vec<SceneFingerprint>,
}

impl GateReport {
    pub(super) fn passes(&self) -> bool {
        false
    }
}

pub(super) fn parse_baseline(
    _json: &serde_json::Value,
) -> Result<Vec<SceneFingerprint>, PlaygroundError> {
    Ok(Vec::new())
}

pub(super) fn parse_observed(
    _json: &serde_json::Value,
) -> Result<Vec<SceneFingerprint>, PlaygroundError> {
    Ok(Vec::new())
}

pub(super) fn compare_fingerprints(
    _protected: &[SceneFingerprint],
    _observed: &[SceneFingerprint],
) -> Result<GateReport, PlaygroundError> {
    Ok(GateReport::default())
}

pub(super) fn render_report(_report: &GateReport, _baseline: &str, _observed: &str) -> String {
    String::new()
}
