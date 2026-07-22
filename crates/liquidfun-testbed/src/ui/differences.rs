//! Canonical semantic difference presentation over immutable comparison input.

use liquidfun_differential::{ComparisonEntry, ComparisonModel, ComparisonState};
use liquidfun_test_protocol::{DebugPrimitiveKey, MathProbePolicyPath};

use super::viewport::Camera;

/// Presentation-only comparison arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMode {
    /// Both backends share one viewport.
    Overlay,
    /// Both backends use synchronized adjacent viewports.
    SideBySide,
    /// One available backend uses the full semantic viewport.
    SingleBackend,
}

/// Closed backend availability used to choose the initial presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendAvailability {
    /// Native Rust and the pinned oracle are both available.
    Both,
    /// Only one backend is available for diagnostics.
    Single,
}

/// Explicit presentation sort. Canonical order always remains available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferenceSort {
    /// Stable canonical semantic-path order.
    CanonicalSemanticPath,
}

/// Mode-independent semantic projection used by every visual arrangement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProjection {
    paths: Box<[Box<str>]>,
    policies: Box<[Option<MathProbePolicyPath>]>,
    primitive_keys: Box<[Option<DebugPrimitiveKey>]>,
}

impl SemanticProjection {
    /// Returns all canonical semantic paths, including exact matches used by overlays.
    #[must_use]
    pub fn paths(&self) -> &[Box<str>] {
        &self.paths
    }
}

/// Immutable canonical entries plus local focus and camera presentation.
#[derive(Debug)]
pub struct DifferenceList<'a> {
    canonical_entries: Box<[&'a ComparisonEntry]>,
    differences: Box<[&'a ComparisonEntry]>,
    mode: ComparisonMode,
    sort: DifferenceSort,
    camera: Camera,
    focused_index: usize,
}

impl<'a> DifferenceList<'a> {
    /// Creates a canonical list, defaulting to overlay when both backends exist.
    #[must_use]
    pub fn new(
        model: &'a ComparisonModel,
        camera: Camera,
        availability: BackendAvailability,
    ) -> Self {
        let mut entries = model.entries().iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.semantic_path().cmp(right.semantic_path()));
        let differences = entries
            .iter()
            .copied()
            .filter(|entry| entry.state() != ComparisonState::ExactMatch)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            canonical_entries: entries.into_boxed_slice(),
            differences,
            mode: if availability == BackendAvailability::Both {
                ComparisonMode::Overlay
            } else {
                ComparisonMode::SingleBackend
            },
            sort: DifferenceSort::CanonicalSemanticPath,
            camera,
            focused_index: 0,
        }
    }

    /// Returns the current presentation arrangement.
    #[must_use]
    pub const fn mode(&self) -> ComparisonMode {
        self.mode
    }

    /// Changes only the visual arrangement.
    pub const fn set_mode(&mut self, mode: ComparisonMode) {
        self.mode = mode;
    }

    /// Returns the synchronized camera shared by all visible viewports.
    #[must_use]
    pub const fn camera(&self) -> Camera {
        self.camera
    }

    /// Returns the explicit active sort.
    #[must_use]
    pub const fn sort(&self) -> DifferenceSort {
        self.sort
    }

    /// Returns every immutable entry in canonical path order.
    #[must_use]
    pub fn entries(&self) -> &[&'a ComparisonEntry] {
        &self.differences
    }

    /// Returns mode-independent paths, policies, and stable focus keys.
    #[must_use]
    pub fn semantic_projection(&self) -> SemanticProjection {
        SemanticProjection {
            paths: self
                .canonical_entries
                .iter()
                .map(|entry| Box::<str>::from(entry.semantic_path()))
                .collect(),
            policies: self
                .canonical_entries
                .iter()
                .map(|entry| entry.maybe_policy_path())
                .collect(),
            primitive_keys: self
                .canonical_entries
                .iter()
                .map(|entry| entry.maybe_primitive_key().cloned())
                .collect(),
        }
    }

    /// Confirms the visible order is canonical semantic-path order.
    #[must_use]
    pub fn is_canonical_path_ordered(&self) -> bool {
        self.differences
            .windows(2)
            .all(|pair| pair[0].semantic_path() <= pair[1].semantic_path())
    }

    /// Wraps focus to the next visible entry.
    pub fn focus_next(&mut self) {
        if !self.differences.is_empty() {
            self.focused_index = (self.focused_index + 1) % self.differences.len();
        }
    }

    /// Wraps focus to the previous visible entry.
    pub fn focus_previous(&mut self) {
        if !self.differences.is_empty() {
            self.focused_index =
                (self.focused_index + self.differences.len() - 1) % self.differences.len();
        }
    }

    /// Announces focused position without exposing unbounded values.
    #[must_use]
    pub fn focused_announcement(&self) -> String {
        if self.differences.is_empty() {
            return "No differences at this checkpoint".to_owned();
        }
        format!(
            "Difference {} of {}",
            self.focused_index + 1,
            self.differences.len()
        )
    }
}

/// Redundant visual encoding for one comparison state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifferenceVisualCue {
    marker: &'static str,
    label: &'static str,
    color: &'static str,
    stroke: &'static str,
    opacity_percent: u8,
    focused_halo: bool,
}

impl DifferenceVisualCue {
    #[must_use]
    pub const fn marker(self) -> &'static str {
        self.marker
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        self.label
    }

    #[must_use]
    pub const fn color(self) -> &'static str {
        self.color
    }

    #[must_use]
    pub const fn stroke(self) -> &'static str {
        self.stroke
    }

    #[must_use]
    pub const fn opacity_percent(self) -> u8 {
        self.opacity_percent
    }

    #[must_use]
    pub const fn focused_halo(self) -> bool {
        self.focused_halo
    }
}

/// Returns the exact icon/text/color/stroke contract for one state.
#[must_use]
pub const fn visual_cue(state: ComparisonState) -> DifferenceVisualCue {
    match state {
        ComparisonState::ExactMatch => DifferenceVisualCue {
            marker: "✓",
            label: "Exact match",
            color: "#3FB950",
            stroke: "solid",
            opacity_percent: 35,
            focused_halo: false,
        },
        ComparisonState::WithinPolicy => DifferenceVisualCue {
            marker: "◇",
            label: "Within policy",
            color: "#D29922",
            stroke: "solid",
            opacity_percent: 100,
            focused_halo: false,
        },
        ComparisonState::PhysicsMismatch => DifferenceVisualCue {
            marker: "×",
            label: "Physics mismatch",
            color: "#F85149",
            stroke: "solid",
            opacity_percent: 100,
            focused_halo: true,
        },
        ComparisonState::RustOnly => DifferenceVisualCue {
            marker: "R",
            label: "Rust-only",
            color: "#FF8C42",
            stroke: "solid",
            opacity_percent: 100,
            focused_halo: false,
        },
        ComparisonState::OracleOnly => DifferenceVisualCue {
            marker: "O",
            label: "Oracle-only",
            color: "#A371F7",
            stroke: "dashed",
            opacity_percent: 100,
            focused_halo: false,
        },
    }
}
