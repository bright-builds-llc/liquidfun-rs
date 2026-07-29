use super::{Deserialize, FloatBits, ScenarioId, Serialize, TransformBits, Vec2Bits};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidBodyKind {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidFixtureShape {
    Circle {
        center: Vec2Bits,
        radius_bits: FloatBits,
    },
    Polygon {
        vertices: Box<[Vec2Bits]>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFilterBits {
    category_bits: u16,
    mask_bits: u16,
    group_index: i16,
}

impl RigidFilterBits {
    #[must_use]
    pub const fn new(category_bits: u16, mask_bits: u16, group_index: i16) -> Self {
        Self {
            category_bits,
            mask_bits,
            group_index,
        }
    }

    #[must_use]
    pub const fn category_bits(self) -> u16 {
        self.category_bits
    }

    #[must_use]
    pub const fn mask_bits(self) -> u16 {
        self.mask_bits
    }

    #[must_use]
    pub const fn group_index(self) -> i16 {
        self.group_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidBodyDeclaration {
    pub(in crate::scenario::rigid_world) body_id: ScenarioId,
    pub(in crate::scenario::rigid_world) body_kind: RigidBodyKind,
    pub(in crate::scenario::rigid_world) transform: TransformBits,
    pub(in crate::scenario::rigid_world) active: bool,
}

impl RigidBodyDeclaration {
    #[must_use]
    pub const fn body_id(&self) -> &ScenarioId {
        &self.body_id
    }

    #[must_use]
    pub const fn body_kind(&self) -> RigidBodyKind {
        self.body_kind
    }

    #[must_use]
    pub const fn transform(&self) -> TransformBits {
        self.transform
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidFixtureDeclaration {
    pub(in crate::scenario::rigid_world) fixture_id: ScenarioId,
    pub(in crate::scenario::rigid_world) owner_body_id: ScenarioId,
    pub(in crate::scenario::rigid_world) shape: RigidFixtureShape,
    pub(in crate::scenario::rigid_world) density_bits: FloatBits,
    pub(in crate::scenario::rigid_world) friction_bits: FloatBits,
    pub(in crate::scenario::rigid_world) restitution_bits: FloatBits,
    pub(in crate::scenario::rigid_world) sensor: bool,
    pub(in crate::scenario::rigid_world) filter: RigidFilterBits,
}

impl RigidFixtureDeclaration {
    #[must_use]
    pub const fn fixture_id(&self) -> &ScenarioId {
        &self.fixture_id
    }

    #[must_use]
    pub const fn owner_body_id(&self) -> &ScenarioId {
        &self.owner_body_id
    }

    #[must_use]
    pub const fn shape(&self) -> &RigidFixtureShape {
        &self.shape
    }

    #[must_use]
    pub const fn density_bits(&self) -> FloatBits {
        self.density_bits
    }

    #[must_use]
    pub const fn friction_bits(&self) -> FloatBits {
        self.friction_bits
    }

    #[must_use]
    pub const fn restitution_bits(&self) -> FloatBits {
        self.restitution_bits
    }

    #[must_use]
    pub const fn sensor(&self) -> bool {
        self.sensor
    }

    #[must_use]
    pub const fn filter(&self) -> RigidFilterBits {
        self.filter
    }
}
