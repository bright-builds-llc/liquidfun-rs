use std::error::Error;
use std::fmt;

use crate::ObjectKind;

/// A failure to resolve an opaque object handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandleError {
    /// The world was poisoned by a panic during a step hook.
    WorldPoisoned,
    /// The handle belongs to a different world.
    WrongWorld,
    /// The handle or related particle object belongs to a different particle system.
    WrongParticleSystem,
    /// The referenced object was destroyed or its slot has since been reused.
    StaleOrDestroyed,
    /// A particle is marked for destruction and no longer accepts ordinary access.
    PendingDelete,
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
            Self::WorldPoisoned => formatter.write_str("world is poisoned by a prior hook panic"),
            Self::WrongWorld => formatter.write_str("handle belongs to a different world"),
            Self::WrongParticleSystem => {
                formatter.write_str("handle belongs to a different particle system")
            }
            Self::StaleOrDestroyed => formatter.write_str("handle is stale or destroyed"),
            Self::PendingDelete => formatter.write_str("particle is pending destruction"),
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

/// A failure to allocate or store a world-owned object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArenaInsertError {
    /// The world was poisoned by a panic during a step hook.
    WorldPoisoned,
    /// Every configured slot is occupied.
    CapacityExceeded {
        /// The configured maximum number of slots.
        limit: usize,
    },
    /// Retired generations have made the configured slot space unusable.
    GenerationExhausted,
    /// The world has issued every semantic diagnostic identity.
    DiagnosticIdExhausted,
}

impl fmt::Display for ArenaInsertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldPoisoned => formatter.write_str("world is poisoned by a prior hook panic"),
            Self::CapacityExceeded { limit } => {
                write!(formatter, "arena capacity of {limit} objects is exhausted")
            }
            Self::GenerationExhausted => {
                formatter.write_str("arena slot generations are exhausted")
            }
            Self::DiagnosticIdExhausted => {
                formatter.write_str("world diagnostic identity space is exhausted")
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
