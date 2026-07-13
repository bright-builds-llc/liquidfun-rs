//! Closed protocol-to-engine value mapping.

use super::{
    BodyMassData, BodyType, CircleShape, FeatureKind, FilterData, FixtureDef, FloatBits,
    NativeRigidWorldError, PolygonShape, RigidBodyDeclaration, RigidBodyKind, RigidContactFeature,
    RigidExpectedCheckpoint, RigidFeatureKind, RigidFilterBits, RigidFixtureDeclaration,
    RigidFixtureShape, RigidWorldActionRecord, RigidWorldTimeline, RigidWorldWitness, ScenarioId,
    Shape, TransformBits, Vec2, Vec2Bits,
};

pub(super) fn native_body_mass_data(
    mass_bits: FloatBits,
    center: Vec2Bits,
    inertia_bits: FloatBits,
) -> Result<BodyMassData, liquidfun::BodyMassDataError> {
    BodyMassData::new(mass_bits.to_f32(), vec2(center), inertia_bits.to_f32())
}

pub(super) fn body_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a RigidBodyDeclaration, NativeRigidWorldError> {
    timeline
        .bodies()
        .iter()
        .find(|declaration| declaration.body_id() == id)
        .ok_or_else(|| action_error(action, format!("missing declaration for body `{id}`")))
}

pub(super) fn fixture_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a RigidFixtureDeclaration, NativeRigidWorldError> {
    timeline
        .fixtures()
        .iter()
        .find(|declaration| declaration.fixture_id() == id)
        .ok_or_else(|| action_error(action, format!("missing declaration for fixture `{id}`")))
}

pub(super) fn fixture_definition(
    declaration: &RigidFixtureDeclaration,
) -> Result<FixtureDef, String> {
    let shape = match declaration.shape() {
        RigidFixtureShape::Circle {
            center,
            radius_bits,
        } => Shape::from(
            CircleShape::new(vec2(*center), radius_bits.to_f32())
                .map_err(|error| error.to_string())?,
        ),
        RigidFixtureShape::Polygon { vertices } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            Shape::from(PolygonShape::new(&vertices).map_err(|error| error.to_string())?)
        }
    };
    FixtureDef::new(
        shape,
        declaration.density_bits().to_f32(),
        declaration.friction_bits().to_f32(),
        declaration.restitution_bits().to_f32(),
        declaration.sensor(),
        filter_data(declaration.filter()),
    )
    .map_err(|error| error.to_string())
}

pub(super) fn action_error(
    action: &RigidWorldActionRecord,
    message: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Action {
        action_id: action.action_id().as_str().into(),
        message: message.to_string().into(),
    }
}

pub(super) fn declaration_error(
    checkpoint: &RigidExpectedCheckpoint,
    message: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Declaration {
        checkpoint_id: checkpoint.checkpoint_id().as_str().into(),
        message: message.to_string().into(),
    }
}

pub(super) fn checked_u32(value: usize, field: &'static str) -> Result<u32, NativeRigidWorldError> {
    u32::try_from(value).map_err(|_| NativeRigidWorldError::Declaration {
        checkpoint_id: field.into(),
        message: "value exceeded the protocol representation".into(),
    })
}

pub(super) const fn body_type(kind: RigidBodyKind) -> BodyType {
    match kind {
        RigidBodyKind::Static => BodyType::Static,
        RigidBodyKind::Kinematic => BodyType::Kinematic,
        RigidBodyKind::Dynamic => BodyType::Dynamic,
    }
}

pub(super) const fn rigid_body_kind(kind: BodyType) -> RigidBodyKind {
    match kind {
        BodyType::Static => RigidBodyKind::Static,
        BodyType::Kinematic => RigidBodyKind::Kinematic,
        BodyType::Dynamic => RigidBodyKind::Dynamic,
    }
}

pub(super) const fn body_created_witness(kind: RigidBodyKind) -> RigidWorldWitness {
    match kind {
        RigidBodyKind::Static => RigidWorldWitness::StaticBodyCreated,
        RigidBodyKind::Kinematic => RigidWorldWitness::KinematicBodyCreated,
        RigidBodyKind::Dynamic => RigidWorldWitness::DynamicBodyCreated,
    }
}

pub(super) const fn filter_data(filter: RigidFilterBits) -> FilterData {
    FilterData::new(
        filter.category_bits(),
        filter.mask_bits(),
        filter.group_index(),
    )
}

pub(super) const fn rigid_filter(filter: FilterData) -> RigidFilterBits {
    RigidFilterBits::new(
        filter.category_bits(),
        filter.mask_bits(),
        filter.group_index(),
    )
}

pub(super) fn vec2(bits: Vec2Bits) -> Vec2 {
    Vec2::new(bits.x_bits.to_f32(), bits.y_bits.to_f32())
}

pub(super) fn vec2_bits(value: Vec2) -> Vec2Bits {
    Vec2Bits {
        x_bits: FloatBits::from_f32(value.x),
        y_bits: FloatBits::from_f32(value.y),
    }
}

pub(super) fn transform_bits(position: Vec2, angle: f32) -> TransformBits {
    TransformBits {
        position: vec2_bits(position),
        angle_bits: FloatBits::from_f32(angle),
    }
}

pub(super) const fn feature(value: liquidfun::collision::ContactFeatureId) -> RigidContactFeature {
    RigidContactFeature {
        index_a: value.index_a(),
        index_b: value.index_b(),
        kind_a: feature_kind(value.kind_a()),
        kind_b: feature_kind(value.kind_b()),
    }
}

pub(super) const fn feature_kind(kind: FeatureKind) -> RigidFeatureKind {
    match kind {
        FeatureKind::Vertex => RigidFeatureKind::Vertex,
        FeatureKind::Face => RigidFeatureKind::Face,
    }
}
