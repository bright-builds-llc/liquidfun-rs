//! Pure accessibility metadata and bounded focused-difference announcements.

const MAXIMUM_ANNOUNCEMENT_FIELD_BYTES: usize = 256;

/// Exact accessibility measurements enforced by the UI contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccessibilityContract {
    minimum_target_pixels: u16,
    focus_ring_pixels: u16,
    focus_contrast_ratio: f32,
    normal_text_contrast_ratio: f32,
    maximum_transition_millis: u16,
    flashing_allowed: bool,
}

/// Approved keyboard, contrast, target, and motion contract.
pub const ACCESSIBILITY_CONTRACT: AccessibilityContract = AccessibilityContract {
    minimum_target_pixels: 44,
    focus_ring_pixels: 2,
    focus_contrast_ratio: 3.0,
    normal_text_contrast_ratio: 4.5,
    maximum_transition_millis: 200,
    flashing_allowed: false,
};

/// Normal-chrome keyboard order; modal surfaces replace this order while open.
pub const NORMAL_FOCUS_ORDER: [&str; 6] = [
    "App bar",
    "Scenario browser",
    "Simulation viewport",
    "Run controls",
    "Inspector",
    "About & provenance",
];

/// Selectable/copyable text without an effectful clipboard dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectableValue<'a>(&'a str);

impl<'a> SelectableValue<'a> {
    /// Creates a selectable view over a bounded semantic value.
    #[must_use]
    pub const fn new(value: &'a str) -> Self {
        Self(value)
    }

    /// Returns the exact text supplied to a platform clipboard adapter.
    #[must_use]
    pub const fn copy_text(self) -> &'a str {
        self.0
    }

    /// Confirms the value is selectable independently of hover/tooltips.
    #[must_use]
    pub const fn is_selectable(self) -> bool {
        true
    }
}

impl AccessibilityContract {
    #[must_use]
    pub const fn minimum_target_pixels(self) -> u16 {
        self.minimum_target_pixels
    }

    #[must_use]
    pub const fn focus_ring_pixels(self) -> u16 {
        self.focus_ring_pixels
    }

    #[must_use]
    pub const fn focus_contrast_ratio(self) -> f32 {
        self.focus_contrast_ratio
    }

    #[must_use]
    pub const fn normal_text_contrast_ratio(self) -> f32 {
        self.normal_text_contrast_ratio
    }

    #[must_use]
    pub const fn maximum_transition_millis(self) -> u16 {
        self.maximum_transition_millis
    }

    #[must_use]
    pub const fn flashing_allowed(self) -> bool {
        self.flashing_allowed
    }
}

/// Builds one concise, bounded mismatch announcement.
///
/// # Errors
///
/// Returns an error for empty, control-bearing, oversized, or invalid ordinal input.
pub fn focused_difference_announcement(
    kind: &str,
    ordinal: usize,
    total: usize,
    semantic_path: &str,
    policy: &str,
    rust_present: bool,
    oracle_present: bool,
) -> Result<String, AccessibilityError> {
    if ordinal == 0 || ordinal > total || total == 0 {
        return Err(AccessibilityError);
    }
    for value in [kind, semantic_path, policy] {
        if value.is_empty()
            || value.len() > MAXIMUM_ANNOUNCEMENT_FIELD_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(AccessibilityError);
        }
    }
    Ok(format!(
        "{kind}, difference {ordinal} of {total}, {semantic_path}, policy {policy}, Rust value {}, oracle value {}.",
        presence(rust_present),
        presence(oracle_present)
    ))
}

const fn presence(present: bool) -> &'static str {
    if present { "present" } else { "absent" }
}

/// Bounded accessibility projection error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("focused difference announcement is invalid")]
pub struct AccessibilityError;
