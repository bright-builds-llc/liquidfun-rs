//! Searchable keyboard catalog projection keyed only by stable identity.

#![allow(
    missing_docs,
    reason = "closed badge and row accessors use the catalog UI vocabulary"
)]

use liquidfun_test_protocol::{
    CatalogEvidenceDisposition, ResolveRequest, ScenarioCatalog, ScenarioConsumer, resolve_catalog,
};

use crate::app::CatalogSelection;

const MAXIMUM_QUERY_BYTES: usize = 256;
const MINIMUM_ROW_TARGET: u16 = 44;

/// Stable browser boundary failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ScenarioBrowserError {
    /// Search text exceeded bounds or contained control characters.
    #[error("scenario search text is invalid")]
    InvalidSearch,
    /// A reviewed definition lacked required presentation metadata.
    #[error("scenario catalog cannot be projected")]
    InvalidCatalog,
}

/// Whether a scenario accepts an exact generator seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedSupport {
    /// Selection must not include a seed.
    NamedOnly,
    /// Selection must include a seed.
    Required,
}

impl SeedSupport {
    /// Returns concise visible metadata.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NamedOnly => "Seed unavailable",
            Self::Required => "Seed required",
        }
    }
}

/// Explicit Rust, oracle, and visual eligibility badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioEligibilityBadges {
    rust: bool,
    oracle: bool,
    visual: bool,
}

impl ScenarioEligibilityBadges {
    #[must_use]
    pub const fn rust(self) -> bool {
        self.rust
    }

    #[must_use]
    pub const fn oracle(self) -> bool {
        self.oracle
    }

    #[must_use]
    pub const fn visual(self) -> bool {
        self.visual
    }
}

/// One actionable, accessible catalog row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioRow {
    display_title: Box<str>,
    category: Box<str>,
    selection: CatalogSelection,
    seed_support: SeedSupport,
    eligibility: ScenarioEligibilityBadges,
}

impl ScenarioRow {
    #[must_use]
    pub fn display_title(&self) -> &str {
        &self.display_title
    }

    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }

    #[must_use]
    pub const fn selection(&self) -> &CatalogSelection {
        &self.selection
    }

    #[must_use]
    pub const fn seed_support(&self) -> SeedSupport {
        self.seed_support
    }

    #[must_use]
    pub const fn eligibility(&self) -> ScenarioEligibilityBadges {
        self.eligibility
    }

    #[must_use]
    pub const fn minimum_target_height(&self) -> u16 {
        MINIMUM_ROW_TARGET
    }
}

/// Pure query, focus, and keyboard selection state over catalog rows.
#[derive(Debug, Clone, Default)]
pub struct ScenarioBrowser {
    all_rows: Vec<ScenarioRow>,
    visible_rows: Vec<ScenarioRow>,
    query: String,
    maybe_focused_index: Option<usize>,
}

impl ScenarioBrowser {
    /// Projects the checked typed registry into stable browser rows.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioBrowserError`] when required reviewed metadata is absent.
    pub fn from_catalog(catalog: &ScenarioCatalog) -> Result<Self, ScenarioBrowserError> {
        let rows = catalog
            .definitions()
            .iter()
            .zip(catalog.mappings())
            .map(|(definition, mapping)| {
                let metadata = definition
                    .metadata()
                    .ok_or(ScenarioBrowserError::InvalidCatalog)?;
                let category = metadata
                    .tags()
                    .first()
                    .ok_or(ScenarioBrowserError::InvalidCatalog)?;
                let settings = metadata.default_settings();
                let named_request = ResolveRequest::new(definition.slug().clone(), None, settings);
                let seeded_request =
                    ResolveRequest::new(definition.slug().clone(), Some(0), settings);
                let named = resolve_catalog(catalog.definitions(), &named_request).is_ok();
                let seeded = resolve_catalog(catalog.definitions(), &seeded_request).is_ok();
                let seed_support = match (named, seeded) {
                    (true, false) => SeedSupport::NamedOnly,
                    (false, true) => SeedSupport::Required,
                    _ => return Err(ScenarioBrowserError::InvalidCatalog),
                };
                let oracle = matches!(
                    mapping.evidence_disposition(),
                    CatalogEvidenceDisposition::Oracle { .. }
                ) || !mapping.upstream_corpus_ids().is_empty();
                Ok(ScenarioRow {
                    display_title: definition.display_title().into(),
                    category: category.as_str().into(),
                    selection: CatalogSelection::new(
                        definition.slug().as_str(),
                        definition.scenario_version().get(),
                        None,
                    ),
                    seed_support,
                    eligibility: ScenarioEligibilityBadges {
                        rust: true,
                        oracle,
                        visual: mapping.is_eligible(ScenarioConsumer::Visual),
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            visible_rows: rows.clone(),
            all_rows: rows,
            query: String::new(),
            maybe_focused_index: None,
        })
    }

    /// Applies bounded case-insensitive title, slug, and category search.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioBrowserError::InvalidSearch`] for unbounded or control text.
    pub fn set_query(&mut self, query: &str) -> Result<(), ScenarioBrowserError> {
        if query.len() > MAXIMUM_QUERY_BYTES || query.chars().any(char::is_control) {
            return Err(ScenarioBrowserError::InvalidSearch);
        }
        self.query.clear();
        self.query.push_str(query);
        let needle = query.to_lowercase();
        self.visible_rows = self
            .all_rows
            .iter()
            .filter(|row| {
                needle.is_empty()
                    || row.display_title().to_lowercase().contains(&needle)
                    || row.selection.catalog_slug().contains(&needle)
                    || row.category().contains(&needle)
            })
            .cloned()
            .collect();
        self.maybe_focused_index = None;
        Ok(())
    }

    /// Returns filtered rows in canonical registry order.
    #[must_use]
    pub fn visible_rows(&self) -> &[ScenarioRow] {
        &self.visible_rows
    }

    /// Focuses the first visible row.
    pub fn focus_first(&mut self) {
        self.maybe_focused_index = (!self.visible_rows.is_empty()).then_some(0);
    }

    /// Moves focus one row down, wrapping within the filtered list.
    pub fn focus_next(&mut self) {
        let length = self.visible_rows.len();
        if length == 0 {
            self.maybe_focused_index = None;
            return;
        }
        self.maybe_focused_index = Some(
            self.maybe_focused_index
                .map_or(0, |index| (index + 1) % length),
        );
    }

    /// Moves focus one row up, wrapping within the filtered list.
    pub fn focus_previous(&mut self) {
        let length = self.visible_rows.len();
        if length == 0 {
            self.maybe_focused_index = None;
            return;
        }
        self.maybe_focused_index = Some(
            self.maybe_focused_index
                .map_or(0, |index| (index + length - 1) % length),
        );
    }

    /// Selects the focused stable slug/version/seed identity.
    #[must_use]
    pub fn select_focused(&self) -> Option<CatalogSelection> {
        self.maybe_focused_index
            .and_then(|index| self.visible_rows.get(index))
            .map(|row| row.selection.clone())
    }
}
