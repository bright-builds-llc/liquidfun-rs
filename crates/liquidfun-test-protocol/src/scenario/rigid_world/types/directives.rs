use super::{
    CodecError, Deserialize, FloatBits, RigidWorldErrorKind, ScenarioId, Serialize, Vec2Bits,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactDirectiveTarget {
    pub fixture_a_id: ScenarioId,
    pub fixture_b_id: ScenarioId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidPreSolveDirective {
    pub enabled: bool,
    pub maybe_friction_bits: Option<FloatBits>,
    pub maybe_restitution_bits: Option<FloatBits>,
    pub maybe_tangent_speed_bits: Option<FloatBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWakePolicy {
    Wake,
    PreserveSleep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidAabbBits {
    pub lower: Vec2Bits,
    pub upper: Vec2Bits,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFixtureChildSelector {
    pub fixture_id: ScenarioId,
    pub child_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidQueryDirective {
    Continue,
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidQueryDirectiveRule {
    pub target: RigidFixtureChildSelector,
    pub directive: RigidQueryDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidRayDirective {
    Ignore,
    Terminate,
    Continue,
    Clip { fraction_bits: FloatBits },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRayDirectiveRule {
    pub target: RigidFixtureChildSelector,
    pub directive: RigidRayDirective,
}

#[derive(Debug, thiserror::Error)]
pub enum RigidWorldDecodeError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("rigid-world validation failed: {0:?}")]
    Validation(RigidWorldErrorKind),
}

impl RigidWorldDecodeError {
    #[must_use]
    pub const fn rigid_world_kind(&self) -> Option<RigidWorldErrorKind> {
        match self {
            Self::Codec(_) => None,
            Self::Validation(kind) => Some(*kind),
        }
    }
}
