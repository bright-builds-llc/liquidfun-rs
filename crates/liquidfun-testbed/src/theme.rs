//! Exact visual tokens approved for the first testbed experience.

#![allow(
    missing_docs,
    reason = "public token field names are the exact UI-SPEC vocabulary"
)]

/// The only layout spacing values, in logical pixels.
pub const SPACING_VALUES: [u16; 7] = [4, 8, 16, 24, 32, 48, 64];
/// The only font sizes, in logical pixels.
pub const FONT_SIZES: [u16; 4] = [12, 14, 18, 24];
/// The only font weights.
pub const FONT_WEIGHTS: [u16; 2] = [400, 600];

/// Exact semantic typography roles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub size: u16,
    pub weight: u16,
    pub line_height: f32,
}

/// Label, body, section-heading, and display styles in that order.
pub const TYPOGRAPHY_STYLES: [TextStyle; 4] = [
    TextStyle {
        size: 12,
        weight: 400,
        line_height: 1.333,
    },
    TextStyle {
        size: 14,
        weight: 400,
        line_height: 1.5,
    },
    TextStyle {
        size: 18,
        weight: 600,
        line_height: 1.333,
    },
    TextStyle {
        size: 24,
        weight: 600,
        line_height: 1.167,
    },
];

/// Approved named spacing values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpacingScale {
    pub xs: u16,
    pub sm: u16,
    pub md: u16,
    pub lg: u16,
    pub xl: u16,
    pub two_xl: u16,
    pub three_xl: u16,
}

/// One source for every layout spacing choice.
pub const SPACING: SpacingScale = SpacingScale {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
    two_xl: 48,
    three_xl: 64,
};

/// Exact accessible dark palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub dominant: &'static str,
    pub secondary: &'static str,
    pub accent: &'static str,
    pub destructive: &'static str,
    pub primary_text: &'static str,
    pub secondary_text: &'static str,
    pub muted_text: &'static str,
    pub border: &'static str,
    pub hover: &'static str,
    pub success: &'static str,
    pub within_policy: &'static str,
    pub rust_only: &'static str,
    pub oracle_only: &'static str,
    pub informational: &'static str,
}

/// Theme and accessibility measurements used by every shell component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub palette: Palette,
    pub field_radius: u16,
    pub panel_radius: u16,
    pub minimum_target: u16,
    pub focus_ring: u16,
    pub normal_text_minimum_contrast: f32,
    pub large_text_minimum_contrast: f32,
    pub maximum_transition_millis: u16,
}

/// Exact first-loaded dark theme.
pub const DARK_THEME: Theme = Theme {
    palette: Palette {
        dominant: "#0D1117",
        secondary: "#161B22",
        accent: "#58A6FF",
        destructive: "#F85149",
        primary_text: "#F0F6FC",
        secondary_text: "#B1BAC4",
        muted_text: "#8B949E",
        border: "#30363D",
        hover: "#21262D",
        success: "#3FB950",
        within_policy: "#D29922",
        rust_only: "#FF8C42",
        oracle_only: "#A371F7",
        informational: "#39C5CF",
    },
    field_radius: 4,
    panel_radius: 8,
    minimum_target: 44,
    focus_ring: 2,
    normal_text_minimum_contrast: 4.5,
    large_text_minimum_contrast: 3.0,
    maximum_transition_millis: 200,
};
