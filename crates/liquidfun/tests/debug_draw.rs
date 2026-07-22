//! Public renderer-neutral debug-primitive contract coverage.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, DebugCollectionError, DebugCollectionResource, DebugDrawLimits,
    DebugDrawOptions, DebugLayer, DebugPrimitive, DebugPrimitiveSink, DistanceJointDef, FixtureDef,
    NoDecisionHook, ParticleDef, ParticleSystemDef, StepConfiguration, StepLimits, World,
};

fn circle_fixture(world: &mut World, body: liquidfun::BodyId) -> liquidfun::FixtureId {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, 1.0).expect("test circle geometry should be valid"),
    );
    let definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture should be valid");
    world
        .create_fixture(body, &definition)
        .expect("test fixture should fit")
}

fn populated_world() -> World {
    let mut world = World::new().expect("world key should remain available");
    let static_body = world
        .create_body(&BodyDef::default())
        .expect("static body should fit");
    circle_fixture(&mut world, static_body);
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.75, 0.0), 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit");
    circle_fixture(&mut world, dynamic_body);
    let joint = DistanceJointDef::new(static_body, dynamic_body)
        .expect("distance joint definition should be valid")
        .with_collide_connected(true);
    world
        .create_joint(joint.into())
        .expect("distance joint should fit");
    let system = world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("particle system should fit");
    for position in [Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)] {
        let receipt = world
            .create_particle_with_def(
                system,
                None,
                &ParticleDef::default()
                    .with_position(position)
                    .expect("particle position should be finite"),
            )
            .expect("particle should fit");
        assert!(receipt.destruction_occurrences().is_empty());
    }
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("step configuration should be valid"),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("contact-producing step should succeed");
    world
}

#[test]
fn collection_is_complete_stable_finite_and_semantically_keyed() {
    // Arrange
    let world = populated_world();

    // Act
    let first = world
        .collect_debug_primitives(DebugDrawOptions::all())
        .expect("reviewed collection should fit");
    let second = world
        .collect_debug_primitives(DebugDrawOptions::all())
        .expect("repeated reviewed collection should fit");

    // Assert
    assert_eq!(first, second);
    for layer in [
        DebugLayer::Shapes,
        DebugLayer::Joints,
        DebugLayer::Contacts,
        DebugLayer::ContactNormals,
        DebugLayer::Particles,
        DebugLayer::ParticleContacts,
        DebugLayer::BroadPhase,
        DebugLayer::CentersOfMass,
        DebugLayer::Labels,
    ] {
        assert!(
            first
                .primitives()
                .iter()
                .any(|primitive| primitive.layer() == layer),
            "missing {layer:?} layer"
        );
    }
    assert!(first.primitives().iter().all(primitive_is_finite));
    let rendered = format!("{first:?}");
    for private_term in ["ProxyId", "dense", "arena", "slot", "storage"] {
        assert!(!rendered.contains(private_term));
    }
    let broad_phase_ordinals = first
        .primitives()
        .iter()
        .filter(|primitive| primitive.layer() == DebugLayer::BroadPhase)
        .map(|primitive| primitive.key().ordinal())
        .collect::<Vec<_>>();
    assert_eq!(
        broad_phase_ordinals,
        (0..u32::try_from(broad_phase_ordinals.len()).expect("reviewed primitive bounds fit u32"))
            .collect::<Vec<_>>()
    );
}

#[test]
fn exact_bounds_sink_replay_and_fresh_empty_frame_preserve_one_model() {
    // Arrange
    let world = populated_world();
    let collection = world
        .collect_debug_primitives(DebugDrawOptions::all())
        .expect("reviewed collection should fit");
    let (vertices, text_bytes) = collection_counts(collection.primitives());
    let exact = DebugDrawLimits::new(collection.primitives().len(), vertices, text_bytes)
        .expect("observed exact limits should be reviewed");
    let mut sink = KeySink::default();

    // Act
    let exact_collection =
        world.collect_debug_primitives(DebugDrawOptions::all().with_limits(exact));
    collection
        .emit_to(&mut sink)
        .expect("infallible sink should accept the collection");
    let empty = World::new()
        .expect("world key should remain available")
        .collect_debug_primitives(DebugDrawOptions::all())
        .expect("empty collection should fit");
    let rejected = world.collect_debug_primitives(DebugDrawOptions::all().with_limits(
        DebugDrawLimits::new(0, vertices, text_bytes).expect("zero is a reviewed limit"),
    ));

    // Assert
    assert_eq!(
        exact_collection.expect("exact bounds are inclusive"),
        collection
    );
    assert_eq!(
        sink.keys,
        collection
            .primitives()
            .iter()
            .map(DebugPrimitive::key)
            .collect::<Vec<_>>()
    );
    assert!(empty.primitives().is_empty());
    assert_eq!(
        rejected,
        Err(DebugCollectionError::CapacityExceeded {
            resource: DebugCollectionResource::Primitives,
            limit: 0,
        })
    );
}

#[test]
fn public_crate_keeps_debug_model_renderer_free_and_rejects_non_finite_style() {
    // Arrange
    let manifest = include_str!("../Cargo.toml");

    // Act
    let maybe_stroke =
        liquidfun::DebugStroke::new(liquidfun::DebugColor::rgba(255, 255, 255, 255), f32::NAN);

    // Assert
    assert!(maybe_stroke.is_none());
    for renderer in ["macroquad", "wgpu", "winit", "egui"] {
        assert!(!manifest.contains(renderer));
    }
}

#[derive(Default)]
struct KeySink {
    keys: Vec<liquidfun::DebugPrimitiveKey>,
}

impl DebugPrimitiveSink for KeySink {
    type Error = std::convert::Infallible;

    fn push(&mut self, primitive: &DebugPrimitive) -> Result<(), Self::Error> {
        self.keys.push(primitive.key());
        Ok(())
    }
}

fn collection_counts(primitives: &[DebugPrimitive]) -> (usize, usize) {
    primitives
        .iter()
        .fold((0, 0), |(vertices, text), primitive| match primitive {
            DebugPrimitive::Segment { .. } | DebugPrimitive::Arrow { .. } => (vertices + 2, text),
            DebugPrimitive::Polyline {
                vertices: points, ..
            } => (vertices + points.len(), text),
            DebugPrimitive::Label { text: label, .. } => (vertices, text + label.len()),
            _ => (vertices, text),
        })
}

fn primitive_is_finite(primitive: &DebugPrimitive) -> bool {
    match primitive {
        DebugPrimitive::Point {
            position, radius, ..
        } => position.is_valid() && radius.is_finite(),
        DebugPrimitive::Segment { start, end, .. } | DebugPrimitive::Arrow { start, end, .. } => {
            start.is_valid() && end.is_valid()
        }
        DebugPrimitive::Polyline { vertices, .. } => {
            vertices.iter().all(|vertex| vertex.is_valid())
        }
        DebugPrimitive::Circle { center, radius, .. } => center.is_valid() && radius.is_finite(),
        DebugPrimitive::TransformAxes {
            transform, scale, ..
        } => transform.position().is_valid() && scale.is_finite(),
        DebugPrimitive::Aabb { bounds, .. } => {
            bounds.lower_bound().is_valid() && bounds.upper_bound().is_valid()
        }
        DebugPrimitive::Label { position, text, .. } => position.is_valid() && !text.is_empty(),
        _ => false,
    }
}
