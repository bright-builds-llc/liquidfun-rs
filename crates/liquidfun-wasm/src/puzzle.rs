//! Puzzle levels built from a level id and construction-time placements, kept apart from the scene allowlist (Phase 37).
#![cfg_attr(not(test), allow(dead_code))] // Temporary until Plan 37-07 wires newPuzzle; Plan 07 removes this line.

pub(crate) mod goal;

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
    match raw {
        "first-pour" => Some(LevelId::FirstPour),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
