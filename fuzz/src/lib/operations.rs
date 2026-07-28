fn decode_operations(bytes: &[u8], maximum: usize) -> Option<Vec<RawOperation>> {
    let operation_count = bytes.len().div_ceil(RAW_OPERATION_BYTES);
    if operation_count > maximum {
        return None;
    }

    let mut operations = Vec::with_capacity(operation_count);
    for chunk in bytes.chunks(RAW_OPERATION_BYTES) {
        let mut padded = [0_u8; RAW_OPERATION_BYTES];
        padded[..chunk.len()].copy_from_slice(chunk);
        let mut input = Unstructured::new(&padded);
        let Ok(operation) = RawOperation::arbitrary(&mut input) else {
            return None;
        };
        operations.push(operation);
    }
    Some(operations)
}

fn finite_scalar(raw: u32) -> f32 {
    let bounded = u16::try_from(raw % 20_001).unwrap_or_default();
    let signed = i16::try_from(bounded).unwrap_or_default() - 10_000;
    f32::from(signed) / 1_000.0
}

fn positive_scalar(raw: u32) -> f32 {
    let bounded = u16::try_from(raw % 2_000 + 1).unwrap_or_default();
    f32::from(bounded) / 1_000.0
}

fn exercise_shape_operation(operation: RawOperation) {
    let circle_a = CircleShape::new(
        Vec2::new(
            finite_scalar(operation.first),
            finite_scalar(operation.second),
        ),
        positive_scalar(operation.third),
    );
    let circle_b = CircleShape::new(Vec2::ZERO, positive_scalar(operation.fourth));
    let polygon = PolygonShape::box_shape(
        positive_scalar(operation.first),
        positive_scalar(operation.second),
    );
    let (Ok(circle_a), Ok(circle_b), Ok(polygon)) = (circle_a, circle_b, polygon) else {
        return;
    };
    let shape_a = if operation.kind & 1 == 0 {
        Shape::from(circle_a)
    } else {
        Shape::from(polygon)
    };
    let shape_b = Shape::from(circle_b);
    let (Ok(child_a), Ok(child_b)) = (shape_a.child_index(0), shape_b.child_index(0)) else {
        return;
    };
    let transform = Transform::from_position_angle(
        Vec2::new(
            finite_scalar(operation.third),
            finite_scalar(operation.fourth),
        ),
        finite_scalar(operation.first),
    );
    if let Ok(CollisionOutcome::Touching(pair)) = collide_shapes(
        &shape_a,
        child_a,
        Transform::IDENTITY,
        &shape_b,
        child_b,
        transform,
    ) {
        assert!(
            pair.manifold()
                .points()
                .iter()
                .all(|point| point.local_point().is_valid())
        );
    }
}

fn execute_world_operation(
    world: &mut World,
    bodies: &mut Vec<BodyId>,
    fixtures: &mut Vec<FixtureId>,
    operation: RawOperation,
) {
    match operation.kind % 6 {
        0 => {
            let body_type = match operation.first % 3 {
                0 => BodyType::Static,
                1 => BodyType::Kinematic,
                _ => BodyType::Dynamic,
            };
            let definition = BodyDef::new(
                body_type,
                Vec2::new(
                    finite_scalar(operation.second),
                    finite_scalar(operation.third),
                ),
                finite_scalar(operation.fourth),
                operation.kind & 1 == 0,
            );
            if let Ok(definition) = definition
                && let Ok(body) = world.create_body(&definition)
            {
                bodies.push(body);
            }
        }
        1 => {
            let Some(body) = select(bodies, operation.first) else {
                return;
            };
            let circle = CircleShape::new(Vec2::ZERO, positive_scalar(operation.second));
            let Ok(circle) = circle else {
                return;
            };
            let definition = FixtureDef::new(
                Shape::from(circle),
                positive_scalar(operation.third),
                positive_scalar(operation.fourth),
                positive_scalar(operation.first),
                operation.kind & 1 == 0,
                FilterData::default(),
            );
            if let Ok(definition) = definition
                && let Ok(fixture) = world.create_fixture(body, &definition)
            {
                fixtures.push(fixture);
            }
        }
        2 => {
            if let Some(body) = select(bodies, operation.first) {
                let _destroyed = world.destroy_body(body);
            }
        }
        3 => {
            if let Some(fixture) = select(fixtures, operation.first) {
                let _destroyed = world.destroy_fixture(fixture);
            }
        }
        4 => {
            if let Some(body) = select(bodies, operation.first) {
                let _transformed = world.set_body_transform(
                    body,
                    Vec2::new(
                        finite_scalar(operation.second),
                        finite_scalar(operation.third),
                    ),
                    finite_scalar(operation.fourth),
                );
            }
        }
        _ => {
            if let Some(body) = select(bodies, operation.first) {
                let _activated = world.set_body_active(body, operation.second & 1 == 0);
            }
        }
    }
}

fn assert_live_bodies_are_finite(world: &World, bodies: &[BodyId]) {
    for body in bodies {
        if let Ok(snapshot) = world.body_snapshot(*body) {
            assert!(snapshot.position().is_valid());
            assert!(snapshot.angle().is_finite());
        }
    }
}

fn checked_particle_budget(operations: &[RawOperation]) -> Option<usize> {
    operations
        .iter()
        .filter(|operation| operation.kind % 4 == 0)
        .try_fold(0_usize, |total, operation| {
            total.checked_add(usize::try_from(operation.first % 32 + 1).ok()?)
        })
}

fn execute_particle_operation(
    world: &mut World,
    system: ParticleSystemId,
    particles: &mut Vec<liquidfun::ParticleId>,
    operation: RawOperation,
) {
    match operation.kind % 4 {
        0 => {
            let count = operation.first % 32 + 1;
            for ordinal in 0..count {
                let definition = ParticleDef::default()
                    .with_position(Vec2::new(
                        finite_scalar(operation.second.wrapping_add(ordinal)),
                        finite_scalar(operation.third),
                    ))
                    .and_then(|definition| {
                        definition.with_velocity(Vec2::new(
                            finite_scalar(operation.fourth),
                            finite_scalar(ordinal),
                        ))
                    });
                if let Ok(definition) = definition
                    && let Ok(receipt) = world.create_particle_with_def(system, None, &definition)
                {
                    particles.push(receipt.created_particle());
                }
            }
        }
        1 => {
            if let Some(particle) = select(particles, operation.first) {
                let _marked = world.mark_particle_for_destruction(particle);
            }
        }
        2 => {
            let _compacted = world.compact_pending_particles(system);
        }
        _ => {
            let Ok(configuration) = StepConfiguration::new(0.0, 1, 1) else {
                panic!("reviewed zero-duration step became invalid");
            };
            let _report = world.step(configuration, &mut NoDecisionHook, StepLimits::default());
        }
    }
}

fn assert_live_particles_are_finite(world: &World, particles: &[liquidfun::ParticleId]) {
    for particle in particles {
        if let Ok(snapshot) = world.particle_snapshot(*particle) {
            assert!(snapshot.position().is_valid());
            assert!(snapshot.velocity().is_valid());
        }
    }
}

fn create_owned_buffer_system(world: &mut World) -> Option<ParticleSystemId> {
    let lanes = ParticleBufferLanes::new(
        Vec::with_capacity(MAX_GROUPS),
        Vec::with_capacity(MAX_GROUPS),
        Vec::with_capacity(MAX_GROUPS),
        None,
    );
    let buffers = ParticleBufferBundle::fixed(MAX_GROUPS, lanes).ok()?;
    world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .ok()
}

fn execute_group_operation(
    world: &mut World,
    system: ParticleSystemId,
    groups: &mut Vec<ParticleGroupId>,
    operation: RawOperation,
) {
    match operation.kind % 3 {
        0 => {
            let source = ParticleGroupSource::positions(vec![Vec2::new(
                finite_scalar(operation.first),
                finite_scalar(operation.second),
            )]);
            let Ok(source) = source else {
                return;
            };
            let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New);
            if let Ok(group) = world.create_particle_group(system, &recipe) {
                groups.push(group);
            }
        }
        1 => {
            if let Some(group) = select(groups, operation.first)
                && let Ok(view) = world.particle_group_view(group)
            {
                let members = view.member_ids().to_vec();
                for member in members {
                    let _marked = world.mark_particle_for_destruction(member);
                }
                let _compacted = world.compact_pending_particles(system);
                let _destroyed = world.destroy_particle_group(group);
            }
        }
        _ => {
            if let Some(group) = select(groups, operation.first) {
                let _view = world.particle_group_view(group);
            }
        }
    }
}

fn select<T: Copy>(values: &[T], raw: u32) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    let index = usize::try_from(raw).unwrap_or_default() % values.len();
    values.get(index).copied()
}
