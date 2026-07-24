use liquidfun::{
    DebugColor, DebugFill, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveCollection,
    DebugPrimitiveKind, DebugPrimitiveMetadata, DebugStroke,
};
use liquidfun_test_protocol::{
    DebugColorBits, DebugFillBits, DebugLayerName, DebugOwnerId, DebugPrimitiveKey,
    DebugPrimitiveKindName, DebugPrimitiveOrder, DebugPrimitiveRecord, DebugStrokeBits, FloatBits,
    PrimitiveAabb, PrimitiveArrow, PrimitiveCircle, PrimitiveLabel, PrimitivePoint,
    PrimitivePolyline, PrimitiveSegment, PrimitiveTransformAxes, ScenarioId, TransformBits,
    Vec2Bits, WireDebugPrimitive,
};

use crate::SessionBackendError;

use super::super::executor::NativeSession;
use super::capture_failure;

pub(super) fn encode_debug_primitives(
    session: &NativeSession,
    collection: &DebugPrimitiveCollection,
) -> Result<Vec<DebugPrimitiveRecord>, SessionBackendError> {
    collection
        .primitives()
        .iter()
        .map(|primitive| {
            Ok(DebugPrimitiveRecord::new(
                DebugPrimitiveOrder::SourceSignificant,
                encode_primitive(session, primitive)?,
            ))
        })
        .collect()
}

fn encode_primitive(
    session: &NativeSession,
    primitive: &DebugPrimitive,
) -> Result<WireDebugPrimitive, SessionBackendError> {
    let (key, stroke, maybe_fill) = encode_metadata(session, primitive.metadata())?;
    let encoded =
        match primitive {
            DebugPrimitive::Point {
                position, radius, ..
            } => WireDebugPrimitive::Point(PrimitivePoint::new(
                key,
                stroke,
                maybe_fill,
                vec2_bits(*position),
                FloatBits::from_f32(*radius),
            )),
            DebugPrimitive::Segment { start, end, .. } => WireDebugPrimitive::Segment(
                PrimitiveSegment::new(key, stroke, maybe_fill, vec2_bits(*start), vec2_bits(*end)),
            ),
            DebugPrimitive::Polyline {
                vertices, closed, ..
            } => WireDebugPrimitive::Polyline(PrimitivePolyline::new(
                key,
                stroke,
                maybe_fill,
                vertices.iter().copied().map(vec2_bits).collect(),
                *closed,
            )),
            DebugPrimitive::Circle { center, radius, .. } => {
                WireDebugPrimitive::Circle(PrimitiveCircle::new(
                    key,
                    stroke,
                    maybe_fill,
                    vec2_bits(*center),
                    FloatBits::from_f32(*radius),
                ))
            }
            DebugPrimitive::TransformAxes {
                transform, scale, ..
            } => WireDebugPrimitive::TransformAxes(PrimitiveTransformAxes::new(
                key,
                stroke,
                maybe_fill,
                TransformBits {
                    position: vec2_bits(transform.position()),
                    angle_bits: FloatBits::from_f32(transform.rotation().angle()),
                },
                FloatBits::from_f32(*scale),
            )),
            DebugPrimitive::Aabb { bounds, .. } => WireDebugPrimitive::Aabb(PrimitiveAabb::new(
                key,
                stroke,
                maybe_fill,
                vec2_bits(bounds.lower_bound()),
                vec2_bits(bounds.upper_bound()),
            )),
            DebugPrimitive::Arrow { start, end, .. } => WireDebugPrimitive::Arrow(
                PrimitiveArrow::new(key, stroke, maybe_fill, vec2_bits(*start), vec2_bits(*end)),
            ),
            DebugPrimitive::Label { position, text, .. } => WireDebugPrimitive::Label(
                PrimitiveLabel::new(key, stroke, maybe_fill, vec2_bits(*position), text.clone()),
            ),
            _ => return Err(capture_failure()),
        };
    Ok(encoded)
}

fn encode_metadata(
    session: &NativeSession,
    metadata: DebugPrimitiveMetadata,
) -> Result<(DebugPrimitiveKey, DebugStrokeBits, Option<DebugFillBits>), SessionBackendError> {
    let source_key = metadata.key();
    let key = DebugPrimitiveKey::new(
        encode_owner(session, source_key.owner())?,
        encode_layer(source_key.layer()),
        encode_kind(source_key.kind()),
        source_key.child(),
        source_key.ordinal(),
    );
    let stroke = encode_stroke(metadata.stroke())?;
    let maybe_fill = metadata.maybe_fill().map(encode_fill);
    Ok((key, stroke, maybe_fill))
}

fn encode_owner(
    session: &NativeSession,
    owner: DebugOwnerKey,
) -> Result<DebugOwnerId, SessionBackendError> {
    let encoded = match owner {
        DebugOwnerKey::World => DebugOwnerId::World,
        DebugOwnerKey::Body(body_id) => DebugOwnerId::Body(
            session
                .bodies
                .iter()
                .find(|(_, candidate)| *candidate == body_id)
                .map(|(scenario_id, _)| scenario_id.clone())
                .ok_or_else(capture_failure)?,
        ),
        DebugOwnerKey::Fixture(fixture_id) => DebugOwnerId::Fixture(
            fixture_scenario_id(session, fixture_id).ok_or_else(capture_failure)?,
        ),
        DebugOwnerKey::Joint(joint_id) => DebugOwnerId::Joint(
            session
                .joints
                .iter()
                .find(|(_, candidate)| *candidate == joint_id)
                .map(|(scenario_id, _)| scenario_id.clone())
                .ok_or_else(capture_failure)?,
        ),
        DebugOwnerKey::Contact {
            fixtures,
            occurrence,
        } => {
            let fixture_a =
                fixture_scenario_id(session, fixtures[0]).ok_or_else(capture_failure)?;
            let fixture_b =
                fixture_scenario_id(session, fixtures[1]).ok_or_else(capture_failure)?;
            DebugOwnerId::Contact(compound_id(
                "contact",
                &[fixture_a.as_str(), fixture_b.as_str()],
                occurrence,
            )?)
        }
        DebugOwnerKey::ParticleSystem(system_id) => DebugOwnerId::ParticleSystem(
            particle_system_scenario_id(session, system_id).ok_or_else(capture_failure)?,
        ),
        DebugOwnerKey::Particle(particle_id) => DebugOwnerId::Particle(
            session
                .particles
                .iter()
                .find(|(_, _, candidate)| *candidate == particle_id)
                .map(|(scenario_id, _, _)| scenario_id.clone())
                .ok_or_else(capture_failure)?,
        ),
        DebugOwnerKey::ParticleContact {
            system,
            particles,
            occurrence,
        } => {
            let system_id =
                particle_system_scenario_id(session, system).ok_or_else(capture_failure)?;
            let particle_a =
                particle_scenario_id(session, system, particles[0]).ok_or_else(capture_failure)?;
            let particle_b =
                particle_scenario_id(session, system, particles[1]).ok_or_else(capture_failure)?;
            DebugOwnerId::ParticleContact(compound_id(
                "particle-contact",
                &[system_id.as_str(), particle_a.as_str(), particle_b.as_str()],
                occurrence,
            )?)
        }
        _ => return Err(capture_failure()),
    };
    Ok(encoded)
}

fn fixture_scenario_id(
    session: &NativeSession,
    fixture_id: liquidfun::FixtureId,
) -> Option<ScenarioId> {
    session
        .fixtures
        .iter()
        .find(|(_, candidate)| *candidate == fixture_id)
        .map(|(scenario_id, _)| scenario_id.clone())
}

fn particle_system_scenario_id(
    session: &NativeSession,
    system_id: liquidfun::ParticleSystemId,
) -> Option<ScenarioId> {
    session
        .systems
        .iter()
        .find(|(_, candidate)| *candidate == system_id)
        .map(|(scenario_id, _)| scenario_id.clone())
}

fn particle_scenario_id(
    session: &NativeSession,
    system_id: liquidfun::ParticleSystemId,
    particle_id: liquidfun::ParticleId,
) -> Option<ScenarioId> {
    session
        .particles
        .iter()
        .find(|(_, candidate_system, candidate_particle)| {
            *candidate_system == system_id && *candidate_particle == particle_id
        })
        .map(|(scenario_id, _, _)| scenario_id.clone())
}

fn compound_id(
    prefix: &str,
    components: &[&str],
    occurrence: u32,
) -> Result<ScenarioId, SessionBackendError> {
    let mut value = String::from(prefix);
    for component in components {
        value.push('-');
        value.push_str(component);
    }
    value.push('-');
    value.push_str(&occurrence.to_string());
    ScenarioId::new(value).map_err(|_error| capture_failure())
}

const fn encode_layer(layer: DebugLayer) -> DebugLayerName {
    match layer {
        DebugLayer::Shapes => DebugLayerName::Shapes,
        DebugLayer::Joints => DebugLayerName::Joints,
        DebugLayer::Contacts => DebugLayerName::Contacts,
        DebugLayer::ContactNormals => DebugLayerName::ContactNormals,
        DebugLayer::Particles => DebugLayerName::Particles,
        DebugLayer::ParticleContacts => DebugLayerName::ParticleContacts,
        DebugLayer::BroadPhase => DebugLayerName::BroadPhase,
        DebugLayer::CentersOfMass => DebugLayerName::CentersOfMass,
        DebugLayer::Labels => DebugLayerName::Labels,
    }
}

const fn encode_kind(kind: DebugPrimitiveKind) -> DebugPrimitiveKindName {
    match kind {
        DebugPrimitiveKind::Point => DebugPrimitiveKindName::Point,
        DebugPrimitiveKind::Segment => DebugPrimitiveKindName::Segment,
        DebugPrimitiveKind::Polyline => DebugPrimitiveKindName::Polyline,
        DebugPrimitiveKind::Circle => DebugPrimitiveKindName::Circle,
        DebugPrimitiveKind::TransformAxes => DebugPrimitiveKindName::TransformAxes,
        DebugPrimitiveKind::Aabb => DebugPrimitiveKindName::Aabb,
        DebugPrimitiveKind::Arrow => DebugPrimitiveKindName::Arrow,
        DebugPrimitiveKind::Label => DebugPrimitiveKindName::Label,
    }
}

fn encode_stroke(stroke: DebugStroke) -> Result<DebugStrokeBits, SessionBackendError> {
    DebugStrokeBits::new(
        encode_color(stroke.color()),
        FloatBits::from_f32(stroke.width()),
    )
    .map_err(|_error| capture_failure())
}

fn encode_fill(fill: DebugFill) -> DebugFillBits {
    DebugFillBits::new(encode_color(fill.color()))
}

const fn encode_color(color: DebugColor) -> DebugColorBits {
    let [red, green, blue, alpha] = color.components();
    DebugColorBits::rgba(red, green, blue, alpha)
}

fn vec2_bits(value: liquidfun::math::Vec2) -> Vec2Bits {
    Vec2Bits {
        x_bits: FloatBits::from_f32(value.x),
        y_bits: FloatBits::from_f32(value.y),
    }
}
