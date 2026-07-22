//! Responsive layout, panel modality, and focus-return presentation state.

use super::viewport::Camera;
use crate::app::{ShellLayout, ShellLayoutMode};

const MAXIMUM_IDENTITY_BYTES: usize = 256;

/// Responsive panel behavior at one logical window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelBehavior {
    BothVisible,
    InspectorDrawer,
    MutuallyExclusiveDrawers,
    FullWindowSheets,
    WindowTooSmall,
}

/// Pure UI-SPEC projection of the shell layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponsiveLayout {
    shell: ShellLayout,
    panel_behavior: PanelBehavior,
    control_rows: u8,
}

impl ResponsiveLayout {
    /// Computes the exact approved breakpoint behavior.
    #[must_use]
    pub const fn for_window(width: u32, height: u32) -> Self {
        let shell = ShellLayout::for_window(width, height);
        let (panel_behavior, control_rows) = match shell.mode() {
            ShellLayoutMode::Standard => (PanelBehavior::BothVisible, 1),
            ShellLayoutMode::InspectorDrawer => (PanelBehavior::InspectorDrawer, 1),
            ShellLayoutMode::ModalDrawers => (PanelBehavior::MutuallyExclusiveDrawers, 2),
            ShellLayoutMode::FullWindowSheets => (PanelBehavior::FullWindowSheets, 1),
            ShellLayoutMode::WindowTooSmall => (PanelBehavior::WindowTooSmall, 0),
        };
        Self {
            shell,
            panel_behavior,
            control_rows,
        }
    }

    #[must_use]
    pub const fn shell(self) -> ShellLayout {
        self.shell
    }

    #[must_use]
    pub const fn panel_behavior(self) -> PanelBehavior {
        self.panel_behavior
    }

    #[must_use]
    pub const fn control_rows(self) -> u8 {
        self.control_rows
    }

    #[must_use]
    pub const fn compact_notice(self) -> Option<&'static str> {
        match self.panel_behavior {
            PanelBehavior::FullWindowSheets => Some("Compact window — panels open one at a time"),
            _ => None,
        }
    }

    #[must_use]
    pub const fn minimum_window_copy(self) -> Option<(&'static str, &'static str)> {
        match self.panel_behavior {
            PanelBehavior::WindowTooSmall => {
                Some(("Window too small", "Resize to at least 640 × 480"))
            }
            _ => None,
        }
    }

    /// Returns the affordances retained even below the supported minimum.
    #[must_use]
    pub const fn minimum_window_affordances(self) -> &'static [&'static str] {
        match self.panel_behavior {
            PanelBehavior::WindowTooSmall => &["Close", "About & provenance"],
            _ => &[],
        }
    }
}

/// Once-per-session compact-window notice state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompactWindowNotice {
    shown: bool,
}

impl CompactWindowNotice {
    /// Returns the notice once when a compact full-window-sheet layout is first observed.
    pub fn take(&mut self, layout: ResponsiveLayout) -> Option<&'static str> {
        let notice = layout.compact_notice()?;
        if self.shown {
            return None;
        }
        self.shown = true;
        Some(notice)
    }
}

/// Immutable identities that resizing, DPI, and panel changes cannot alter.
#[derive(Debug, Clone, PartialEq)]
pub struct PresentationIdentitySnapshot {
    camera: Camera,
    selection: Box<str>,
    checkpoint: Box<str>,
    controller: Box<str>,
}

/// Local responsive state with semantic identities retained across layout changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponsivePresentation {
    identity: PresentationIdentitySnapshot,
    layout: ResponsiveLayout,
    dpi_scale: f32,
}

impl ResponsivePresentation {
    /// Creates bounded local presentation state.
    ///
    /// # Errors
    ///
    /// Rejects empty, control-bearing, or oversized identity labels.
    pub fn new(
        camera: Camera,
        selection: &str,
        checkpoint: &str,
        controller: &str,
    ) -> Result<Self, LayoutError> {
        for value in [selection, checkpoint, controller] {
            validate_identity(value)?;
        }
        Ok(Self {
            identity: PresentationIdentitySnapshot {
                camera,
                selection: selection.into(),
                checkpoint: checkpoint.into(),
                controller: controller.into(),
            },
            layout: ResponsiveLayout::for_window(1280, 720),
            dpi_scale: 1.0,
        })
    }

    /// Changes only local layout and DPI presentation.
    pub fn resize(&mut self, width: u32, height: u32, dpi_scale: f32) {
        if !dpi_scale.is_finite() || !(0.5..=4.0).contains(&dpi_scale) {
            return;
        }
        self.layout = ResponsiveLayout::for_window(width, height);
        self.dpi_scale = dpi_scale;
    }

    #[must_use]
    pub fn identity_snapshot(&self) -> PresentationIdentitySnapshot {
        self.identity.clone()
    }
}

fn validate_identity(value: &str) -> Result<(), LayoutError> {
    if value.is_empty()
        || value.len() > MAXIMUM_IDENTITY_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(LayoutError);
    }
    Ok(())
}

/// Stable focus identities for modal focus entry and return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusId {
    ScenarioButton,
    ScenarioHeading,
    ScenarioSearch,
    InspectorButton,
    InspectorHeading,
    InspectorDifference,
    SettingsButton,
    SettingsHeading,
    SettingsField,
    AboutButton,
    AboutHeading,
    AboutLink,
    ShortcutHelp,
}

/// Focus state that returns to the modal invoker on close.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FocusReturn {
    maybe_current: Option<FocusId>,
    maybe_invoker: Option<FocusId>,
}

impl FocusReturn {
    /// Moves focus into a newly opened modal surface.
    pub const fn open(&mut self, invoker: FocusId, first_control: FocusId) {
        self.maybe_invoker = Some(invoker);
        self.maybe_current = Some(first_control);
    }

    /// Moves focus within the currently open modal surface without changing its invoker.
    pub const fn move_to(&mut self, control: FocusId) {
        self.maybe_current = Some(control);
    }

    /// Closes the top surface and returns focus to its invoker.
    pub const fn close(&mut self) -> Option<FocusId> {
        let returned = self.maybe_invoker.take();
        self.maybe_current = returned;
        returned
    }

    #[must_use]
    pub const fn current(self) -> Option<FocusId> {
        self.maybe_current
    }
}

/// Bounded responsive presentation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("responsive presentation identity is invalid")]
pub struct LayoutError;
