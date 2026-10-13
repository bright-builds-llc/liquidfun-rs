//! Puzzle levels built from a level id and construction-time placements, kept apart from the scene allowlist (Phase 37).

pub(crate) mod angles;
pub(crate) mod builder;
pub(crate) mod first_pour;
pub(crate) mod goal;
pub(crate) mod hooks;
mod info;
mod inspect;
pub(crate) mod peak;
pub(crate) mod sat;
pub(crate) mod spec;
pub(crate) mod tools;
pub(crate) mod validate;
pub(crate) mod valve;

use self::spec::LevelSpec;

pub use info::PuzzleLevelInfo;
pub use inspect::{PlacementInspection, inspect_puzzle_placements};

/// Allowlisted puzzle level, separate from `SceneId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LevelId {
    /// "First Pour": plain water poured toward a cup.
    FirstPour,
}

/// Every allowlisted level, in catalog order. Mirrored by `PUZZLE_IDS` in
/// `web/src/catalog/puzzles.ts`.
pub(crate) const LEVEL_IDS: [LevelId; 1] = [LevelId::FirstPour];

impl LevelId {
    /// Stable kebab-case id shared with the web catalog.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FirstPour => "first-pour",
        }
    }
}

/// Parses an allowlisted level id; scene ids are never level ids.
pub(crate) fn parse_level_id(raw: &str) -> Option<LevelId> {
    LEVEL_IDS.into_iter().find(|id| id.as_str() == raw)
}

/// The const spec for an allowlisted level.
pub(crate) const fn level_spec(id: LevelId) -> &'static LevelSpec {
    match id {
        LevelId::FirstPour => &first_pour::FIRST_POUR,
    }
}

#[cfg(test)]
mod api_tests;
#[cfg(test)]
mod materials_tests;
#[cfg(test)]
mod replay;
#[cfg(test)]
mod session_tests;
#[cfg(test)]
mod tests;
