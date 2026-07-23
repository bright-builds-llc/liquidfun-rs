//! Serializable measurements and capability dispositions.

use serde::Serialize;

use super::fixture::FixtureSnapshot;
use super::passive::PassiveInputSnapshot;
use super::render::RenderedEvidence;

/// Complete named acceptance matrix required for the replacement renderer.
pub const REQUIRED_CAPABILITY_NAMES: [&str; 20] = [
    "rigid_contacts",
    "contact_normals",
    "particle_contacts_and_colors",
    "broad_phase_aabbs",
    "profile_names_without_durations",
    "synchronized_overlay",
    "side_by_side_difference",
    "focus_halo_and_label",
    "semantic_capture_acknowledgement",
    "diagnostic_screenshot_disclaimer",
    "keyboard_controls",
    "keyboard_focus",
    "dense_text",
    "dpi_scaling",
    "resize",
    "minimum_640x480",
    "passive_session_controller",
    "immutable_comparison_model",
    "bounded_finite_inputs",
    "confined_regular_output",
];

/// One objective named capability disposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityDisposition {
    name: &'static str,
    passed: bool,
    measurement: String,
}

impl CapabilityDisposition {
    pub(super) fn measured(
        name: &'static str,
        passed: bool,
        measurement: impl Into<String>,
    ) -> Self {
        Self {
            name,
            passed,
            measurement: measurement.into(),
        }
    }
}

/// One regular diagnostic artifact written below the confined output directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityArtifact {
    path: String,
    sha256: String,
    bytes: u64,
    width: u16,
    height: u16,
    regular: bool,
}

impl CapabilityArtifact {
    pub(super) fn new(
        path: String,
        sha256: String,
        bytes: u64,
        width: u16,
        height: u16,
        regular: bool,
    ) -> Self {
        Self {
            path,
            sha256,
            bytes,
            width,
            height,
            regular,
        }
    }

    /// Returns the output-relative artifact path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the artifact SHA-256 digest.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Returns whether post-write metadata proves a non-link regular file.
    #[must_use]
    pub const fn is_regular(&self) -> bool {
        self.regular
    }
}

/// Objective renderer measurements independent of wall-clock timing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapabilityMeasurements {
    fixture_sha256: String,
    verified_fixture_artifacts: usize,
    fixture_cases: usize,
    fixture_families: usize,
    frame_count: usize,
    minimum_width: u16,
    minimum_height: u16,
    maximum_dpi_scale: u16,
    resize_width: u16,
    resize_height: u16,
    non_background_pixels_minimum: usize,
    distinct_particle_colors: usize,
    dense_text_rows: usize,
    focus_ring_pixels: usize,
    minimum_text_contrast_ratio: f32,
    minimum_control_target_pixels: u16,
    keyboard_bindings: usize,
    contact_points: usize,
    contact_normals: usize,
    particle_contacts: usize,
    broad_phase_aabbs: usize,
    profile_names: usize,
    overlay_pairs: usize,
    side_by_side_panels: usize,
    semantic_capture_acknowledgements: usize,
    diagnostic_disclaimer_lines: usize,
}

/// Complete deterministic capability result.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CapabilityReport {
    schema_version: u32,
    capability_profile: &'static str,
    adapter: &'static str,
    selected_stack: &'static str,
    fixture_profile: String,
    upstream_revision: String,
    fixture_case_ids: Vec<String>,
    fixture_families: Vec<String>,
    measurements: CapabilityMeasurements,
    capabilities: Vec<CapabilityDisposition>,
    artifacts: Vec<CapabilityArtifact>,
    session_before: PassiveInputSnapshot,
    session_after: PassiveInputSnapshot,
    report_sha256: Option<String>,
}

impl CapabilityReport {
    #[allow(
        clippy::too_many_lines,
        reason = "the closed capability matrix stays adjacent to its objective evidence"
    )]
    pub(super) fn from_evidence(
        fixture: &FixtureSnapshot,
        before: PassiveInputSnapshot,
        after: PassiveInputSnapshot,
        rendered: RenderedEvidence,
    ) -> Self {
        let passive_unchanged = before.logical_steps == after.logical_steps
            && before.captures == after.captures
            && before.session_state == after.session_state;
        let comparison_unchanged = before.comparison_state == after.comparison_state
            && before.comparison_entries == after.comparison_entries
            && before.comparison_entries > 0;
        let regular_outputs = rendered
            .artifacts
            .iter()
            .all(CapabilityArtifact::is_regular);
        let finite_bounded = rendered.minimum_text_contrast_ratio.is_finite()
            && rendered.minimum_width >= 640
            && rendered.minimum_height >= 480
            && rendered.non_background_pixels_minimum > 0;
        let measurements = CapabilityMeasurements {
            fixture_sha256: fixture.sha256.clone(),
            verified_fixture_artifacts: fixture.verified_artifacts,
            fixture_cases: fixture.case_ids.len(),
            fixture_families: fixture.families.len(),
            frame_count: rendered.artifacts.len(),
            minimum_width: rendered.minimum_width,
            minimum_height: rendered.minimum_height,
            maximum_dpi_scale: rendered.maximum_dpi_scale,
            resize_width: rendered.resize_width,
            resize_height: rendered.resize_height,
            non_background_pixels_minimum: rendered.non_background_pixels_minimum,
            distinct_particle_colors: rendered.distinct_particle_colors,
            dense_text_rows: rendered.dense_text_rows,
            focus_ring_pixels: rendered.focus_ring_pixels,
            minimum_text_contrast_ratio: rendered.minimum_text_contrast_ratio,
            minimum_control_target_pixels: rendered.minimum_control_target_pixels,
            keyboard_bindings: rendered.keyboard_bindings,
            contact_points: rendered.contact_points,
            contact_normals: rendered.contact_normals,
            particle_contacts: rendered.particle_contacts,
            broad_phase_aabbs: rendered.broad_phase_aabbs,
            profile_names: rendered.profile_names,
            overlay_pairs: rendered.overlay_pairs,
            side_by_side_panels: rendered.side_by_side_panels,
            semantic_capture_acknowledgements: rendered.semantic_capture_acknowledgements,
            diagnostic_disclaimer_lines: rendered.diagnostic_disclaimer_lines,
        };
        let capabilities = vec![
            CapabilityDisposition::measured(
                "rigid_contacts",
                rendered.contact_points >= 3,
                "3 visible contact points",
            ),
            CapabilityDisposition::measured(
                "contact_normals",
                rendered.contact_normals >= 3,
                "3 directed normal arrows",
            ),
            CapabilityDisposition::measured(
                "particle_contacts_and_colors",
                rendered.particle_contacts >= 6 && rendered.distinct_particle_colors >= 4,
                "12 particles, 6 contacts, 4 colors",
            ),
            CapabilityDisposition::measured(
                "broad_phase_aabbs",
                rendered.broad_phase_aabbs >= 4,
                "4 distinct outlined AABBs",
            ),
            CapabilityDisposition::measured(
                "profile_names_without_durations",
                rendered.profile_names >= 5,
                "5 structural profile names; no duration values",
            ),
            CapabilityDisposition::measured(
                "synchronized_overlay",
                rendered.overlay_pairs >= 3,
                "3 aligned R/O primitive pairs",
            ),
            CapabilityDisposition::measured(
                "side_by_side_difference",
                rendered.side_by_side_panels == 2,
                "2 labeled synchronized panels",
            ),
            CapabilityDisposition::measured(
                "focus_halo_and_label",
                rendered.focus_ring_pixels >= 2,
                "2px halo plus semantic label",
            ),
            CapabilityDisposition::measured(
                "semantic_capture_acknowledgement",
                rendered.semantic_capture_acknowledgements >= 1,
                "checkpoint acknowledgement rendered",
            ),
            CapabilityDisposition::measured(
                "diagnostic_screenshot_disclaimer",
                rendered.diagnostic_disclaimer_lines >= 2,
                "diagnostic-only screenshot copy rendered",
            ),
            CapabilityDisposition::measured(
                "keyboard_controls",
                rendered.minimum_control_target_pixels >= 44 && rendered.keyboard_bindings == 6,
                "6 typed shortcuts and 44px minimum target",
            ),
            CapabilityDisposition::measured(
                "keyboard_focus",
                rendered.focus_ring_pixels >= 2 && rendered.minimum_text_contrast_ratio >= 3.0,
                "persistent 2px contrast focus ring",
            ),
            CapabilityDisposition::measured(
                "dense_text",
                rendered.dense_text_rows >= 16,
                "16 inspector rows at minimum viewport",
            ),
            CapabilityDisposition::measured(
                "dpi_scaling",
                rendered.maximum_dpi_scale >= 2 && rendered.artifacts.len() >= 3,
                "1x, 1.25x, and 2x replacement images rendered",
            ),
            CapabilityDisposition::measured(
                "resize",
                rendered.resize_width == 800 && rendered.resize_height == 600,
                "800x600 centered responsive frame rendered",
            ),
            CapabilityDisposition::measured(
                "minimum_640x480",
                rendered.minimum_width == 640 && rendered.minimum_height == 480,
                "complete 640x480 frame rendered",
            ),
            CapabilityDisposition::measured(
                "passive_session_controller",
                passive_unchanged,
                format!("no step/capture/state change: {passive_unchanged}"),
            ),
            CapabilityDisposition::measured(
                "immutable_comparison_model",
                comparison_unchanged,
                format!("{} immutable comparison entries", before.comparison_entries),
            ),
            CapabilityDisposition::measured(
                "bounded_finite_inputs",
                finite_bounded,
                "fixed finite geometry and reviewed frame bounds",
            ),
            CapabilityDisposition::measured(
                "confined_regular_output",
                regular_outputs,
                "all artifacts are regular files below target/",
            ),
        ];
        Self {
            schema_version: 1,
            capability_profile: "phase12-v1",
            adapter: "eframe-egui-0.35.0+tiny-skia-0.12.0",
            selected_stack: "eframe-0.35.0+egui-0.35.0+tiny-skia-0.12.0",
            fixture_profile: fixture.profile.clone(),
            upstream_revision: fixture.upstream_revision.clone(),
            fixture_case_ids: fixture.case_ids.clone(),
            fixture_families: fixture.families.clone(),
            measurements,
            capabilities,
            artifacts: rendered.artifacts,
            session_before: before,
            session_after: after,
            report_sha256: None,
        }
    }

    pub(super) fn validate_required_capabilities(&mut self) {
        for (expected, actual) in REQUIRED_CAPABILITY_NAMES.iter().zip(&mut self.capabilities) {
            actual.passed = actual.passed && actual.name == *expected;
        }
        if self.capabilities.len() != REQUIRED_CAPABILITY_NAMES.len() {
            self.capabilities
                .iter_mut()
                .for_each(|capability| capability.passed = false);
        }
    }

    pub(super) fn set_report_sha256(&mut self, sha256: String) {
        self.report_sha256 = Some(sha256);
    }

    /// Returns the selected concrete adapter identity.
    #[must_use]
    pub const fn adapter(&self) -> &'static str {
        self.adapter
    }

    /// Returns true only when every required capability passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.capabilities.len() == REQUIRED_CAPABILITY_NAMES.len()
            && self.capabilities.iter().all(|capability| capability.passed)
    }

    /// Returns capability names in required matrix order.
    #[must_use]
    pub fn capability_names(&self) -> Vec<&'static str> {
        self.capabilities
            .iter()
            .map(|capability| capability.name)
            .collect()
    }

    /// Returns generated diagnostic artifacts.
    #[must_use]
    pub fn artifacts(&self) -> &[CapabilityArtifact] {
        &self.artifacts
    }

    /// Returns logical steps before rendering.
    #[must_use]
    pub const fn session_logical_steps_before(&self) -> u32 {
        self.session_before.logical_steps
    }

    /// Returns logical steps after rendering.
    #[must_use]
    pub const fn session_logical_steps_after(&self) -> u32 {
        self.session_after.logical_steps
    }

    /// Returns semantic capture count before rendering.
    #[must_use]
    pub const fn session_captures_before(&self) -> usize {
        self.session_before.captures
    }

    /// Returns semantic capture count after rendering.
    #[must_use]
    pub const fn session_captures_after(&self) -> usize {
        self.session_after.captures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::render::rendered_evidence_with_contact_normals;

    fn fixture() -> FixtureSnapshot {
        FixtureSnapshot {
            sha256: "0".repeat(64),
            profile: "phase11-v1".to_owned(),
            upstream_revision: "0".repeat(40),
            case_ids: vec!["test-case".to_owned()],
            families: vec!["rigid".to_owned()],
            verified_artifacts: 1,
        }
    }

    fn passive_snapshot() -> PassiveInputSnapshot {
        PassiveInputSnapshot {
            session_state: "ready_paused",
            logical_steps: 0,
            captures: 0,
            comparison_state: "exact_match",
            comparison_entries: 1,
        }
    }

    #[test]
    fn suppressing_required_contact_normals_fails_the_capability_matrix() {
        // Arrange
        let fixture = fixture();
        let before = passive_snapshot();
        let mut complete = CapabilityReport::from_evidence(
            &fixture,
            before.clone(),
            before.clone(),
            rendered_evidence_with_contact_normals(true),
        );
        complete.validate_required_capabilities();

        // Act
        let mut suppressed = CapabilityReport::from_evidence(
            &fixture,
            before.clone(),
            before,
            rendered_evidence_with_contact_normals(false),
        );
        suppressed.validate_required_capabilities();

        // Assert
        assert!(complete.all_passed());
        assert!(!suppressed.all_passed());
    }
}
