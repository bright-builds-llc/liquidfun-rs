use std::error::Error;
use std::fmt;

use crate::ObjectKind;

/// A failure to resolve an opaque object handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandleError {
    /// The handle belongs to a different world.
    WrongWorld,
    /// The referenced object was destroyed or its slot has since been reused.
    StaleOrDestroyed,
    /// An internal heterogeneous lookup received a different handle kind.
    ///
    /// Public typed lookup APIs cannot produce this variant.
    WrongKind {
        /// The object kind required by the lookup.
        expected: ObjectKind,
        /// The object kind carried by the erased internal handle.
        actual: ObjectKind,
    },
}

impl fmt::Display for HandleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongWorld => formatter.write_str("handle belongs to a different world"),
            Self::StaleOrDestroyed => formatter.write_str("handle is stale or destroyed"),
            Self::WrongKind { expected, actual } => {
                write!(
                    formatter,
                    "wrong handle kind: expected {expected}, received {actual}"
                )
            }
        }
    }
}

impl Error for HandleError {}

/// A failure to insert an object into a bounded generational arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArenaInsertError {
    /// Every configured slot is occupied.
    CapacityExceeded {
        /// The configured maximum number of slots.
        limit: usize,
    },
    /// Retired generations have made the configured slot space unusable.
    GenerationExhausted,
}

impl fmt::Display for ArenaInsertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded { limit } => {
                write!(formatter, "arena capacity of {limit} objects is exhausted")
            }
            Self::GenerationExhausted => {
                formatter.write_str("arena slot generations are exhausted")
            }
        }
    }
}

impl Error for ArenaInsertError {}

/// A failure to allocate a process-unique world identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldKeyError {
    /// The process has exhausted the complete world-key space.
    Exhausted,
}

impl fmt::Display for WorldKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted => formatter.write_str("process world-key space is exhausted"),
        }
    }
}

impl Error for WorldKeyError {}
