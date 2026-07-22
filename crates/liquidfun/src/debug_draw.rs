//! Bounded renderer-neutral debug geometry over stable semantic identities.

mod collector;
mod primitive;

pub use collector::{
    DebugCollectionError, DebugCollectionResource, DebugDrawLimits, DebugDrawOptions,
    DebugPrimitiveCollection, DebugPrimitiveSink,
};
pub use primitive::{
    DebugColor, DebugFill, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveKey,
    DebugPrimitiveKind, DebugPrimitiveMetadata, DebugStroke,
};
