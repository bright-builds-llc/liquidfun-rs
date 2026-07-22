//! Local presentation state for the passive desktop shell.

const DEFAULT_ZOOM_PERCENT: u16 = 100;

/// Camera transform owned solely by the visual adapter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPresentation {
    center_x: f32,
    center_y: f32,
    zoom_percent: u16,
}

impl Default for CameraPresentation {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom_percent: DEFAULT_ZOOM_PERCENT,
        }
    }
}

impl CameraPresentation {
    /// Returns the horizontal presentation offset.
    #[must_use]
    pub const fn center_x(&self) -> f32 {
        self.center_x
    }

    /// Returns the vertical presentation offset.
    #[must_use]
    pub const fn center_y(&self) -> f32 {
        self.center_y
    }

    /// Returns the integral zoom percentage displayed in chrome.
    #[must_use]
    pub const fn zoom_percent(&self) -> u16 {
        self.zoom_percent
    }
}

/// Which optional shell panels are visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelPresentation {
    scenario_browser_open: bool,
    inspector_open: bool,
    about_open: bool,
}

impl Default for PanelPresentation {
    fn default() -> Self {
        Self {
            scenario_browser_open: true,
            inspector_open: true,
            about_open: false,
        }
    }
}

impl PanelPresentation {
    /// Returns whether the scenario browser is visible.
    #[must_use]
    pub const fn scenario_browser_open(self) -> bool {
        self.scenario_browser_open
    }

    /// Returns whether the inspector is visible.
    #[must_use]
    pub const fn inspector_open(self) -> bool {
        self.inspector_open
    }

    /// Returns whether the About panel is visible.
    #[must_use]
    pub const fn about_open(self) -> bool {
        self.about_open
    }
}

/// Local panel scroll offsets in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollPresentation {
    catalog: f32,
    inspector: f32,
}

impl ScrollPresentation {
    /// Returns the scenario browser scroll offset.
    #[must_use]
    pub const fn catalog(self) -> f32 {
        self.catalog
    }

    /// Returns the inspector scroll offset.
    #[must_use]
    pub const fn inspector(self) -> f32 {
        self.inspector
    }
}

/// Keyboard focus target within normal product chrome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTarget {
    /// Scenario search field.
    ScenarioSearch,
    /// Stable catalog row identity.
    CatalogRow(CatalogSelection),
    /// Stable semantic observation key.
    SemanticKey(Box<str>),
    /// About and provenance affordance.
    About,
}

/// Stable catalog selection independent of its display title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSelection {
    catalog_slug: Box<str>,
    scenario_version: u32,
    maybe_seed: Option<u64>,
}

impl CatalogSelection {
    /// Creates a selection from already validated catalog identity fields.
    #[must_use]
    pub fn new(
        catalog_slug: impl Into<Box<str>>,
        scenario_version: u32,
        maybe_seed: Option<u64>,
    ) -> Self {
        Self {
            catalog_slug: catalog_slug.into(),
            scenario_version,
            maybe_seed,
        }
    }

    /// Returns the stable catalog slug.
    #[must_use]
    pub fn catalog_slug(&self) -> &str {
        &self.catalog_slug
    }

    /// Returns the stable scenario version.
    #[must_use]
    pub const fn scenario_version(&self) -> u32 {
        self.scenario_version
    }

    /// Returns the exact optional generator seed.
    #[must_use]
    pub const fn maybe_seed(&self) -> Option<u64> {
        self.maybe_seed
    }
}

/// Search and eligibility filters applied only to visible catalog rows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogFilter {
    query: Box<str>,
    visual_only: bool,
}

impl CatalogFilter {
    /// Returns the bounded search query.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns whether non-visual rows are hidden.
    #[must_use]
    pub const fn visual_only(&self) -> bool {
        self.visual_only
    }
}

/// Diagnostic screenshot presentation options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenshotOptions {
    include_chrome: bool,
    scale: ScreenshotScale,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            include_chrome: true,
            scale: ScreenshotScale::One,
        }
    }
}

impl ScreenshotOptions {
    /// Returns whether product chrome is included.
    #[must_use]
    pub const fn include_chrome(self) -> bool {
        self.include_chrome
    }

    /// Returns the requested diagnostic scale.
    #[must_use]
    pub const fn scale(self) -> ScreenshotScale {
        self.scale
    }
}

/// Closed diagnostic screenshot scales.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScreenshotScale {
    /// One logical pixel per output pixel.
    #[default]
    One,
    /// Two output pixels per logical pixel.
    Two,
}

/// Complete mutable state local to the passive shell.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AppState {
    camera: CameraPresentation,
    panels: PanelPresentation,
    scroll: ScrollPresentation,
    maybe_focus: Option<FocusTarget>,
    filters: CatalogFilter,
    maybe_selected_catalog: Option<CatalogSelection>,
    maybe_selected_semantic_key: Option<Box<str>>,
    screenshot: ScreenshotOptions,
}

impl AppState {
    /// Returns the camera presentation.
    #[must_use]
    pub const fn camera(&self) -> &CameraPresentation {
        &self.camera
    }

    /// Returns local panel visibility.
    #[must_use]
    pub const fn panels(&self) -> PanelPresentation {
        self.panels
    }

    /// Returns local panel scroll offsets.
    #[must_use]
    pub const fn scroll(&self) -> ScrollPresentation {
        self.scroll
    }

    /// Returns the current keyboard focus target.
    #[must_use]
    pub const fn focus(&self) -> Option<&FocusTarget> {
        self.maybe_focus.as_ref()
    }

    /// Returns catalog presentation filters.
    #[must_use]
    pub const fn filters(&self) -> &CatalogFilter {
        &self.filters
    }

    /// Returns the selected stable catalog identity.
    #[must_use]
    pub const fn selected_catalog(&self) -> Option<&CatalogSelection> {
        self.maybe_selected_catalog.as_ref()
    }

    /// Returns the selected stable semantic key.
    #[must_use]
    pub fn selected_semantic_key(&self) -> Option<&str> {
        self.maybe_selected_semantic_key.as_deref()
    }

    /// Returns diagnostic screenshot options.
    #[must_use]
    pub const fn screenshot(&self) -> ScreenshotOptions {
        self.screenshot
    }
}
