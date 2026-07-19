use crate::collision::{ChainShape, CircleShape, EdgeShape, PolygonShape};
use crate::identity::{HandleIdentity, Identity, WorldKey};
use crate::math::{Rotation, Transform};

use super::*;

fn circle() -> Shape {
    CircleShape::new(Vec2::ZERO, 1.0)
        .expect("test circle should be valid")
        .into()
}

fn polygon() -> Shape {
    PolygonShape::box_shape(1.0, 2.0)
        .expect("test box should be valid")
        .into()
}

fn edge() -> Shape {
    EdgeShape::new(Vec2::ZERO, Vec2::new(1.0, 0.0))
        .expect("test edge should be valid")
        .into()
}

fn chain() -> Shape {
    ChainShape::open(
        &[Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        None,
        None,
    )
    .expect("test chain should be valid")
    .into()
}

fn positions_source() -> ParticleGroupSource {
    ParticleGroupSource::positions(vec![Vec2::ZERO]).expect("test position should be valid")
}

fn recipe() -> ParticleGroupRecipe {
    ParticleGroupRecipe::new(positions_source(), ParticleGroupDestination::New)
}

fn particle_id(world: WorldKey, slot: usize) -> ParticleId {
    ParticleId::from_identity(Identity::new(world, slot, 0))
}

fn group_id(world: WorldKey) -> ParticleGroupId {
    ParticleGroupId::from_identity(Identity::new(world, 20, 0))
}

fn view_state(world: WorldKey) -> ParticleGroupViewState {
    ParticleGroupViewState {
        id: group_id(world),
        flags: ParticleGroupFlags::SOLID,
        transform: Transform::from_position_angle(Vec2::new(3.0, 4.0), 0.25),
        center: Vec2::new(5.0, 6.0),
        linear_velocity: Vec2::new(7.0, 8.0),
        angular_velocity: 9.0,
        mass: 10.0,
        inertia: 11.0,
    }
}

#[test]
fn filled_source_owns_non_empty_circle_polygon_union() {
    // Arrange
    let shapes = vec![circle(), polygon()];

    // Act
    let source = ParticleGroupSource::filled_shapes(shapes)
        .expect("circle and polygon should form a filled union");

    // Assert
    let ParticleGroupSource::FilledShapes(filled) = source else {
        panic!("filled constructor should select the filled variant");
    };
    assert_eq!(filled.shapes().len(), 2);
}

#[test]
fn stroke_source_owns_one_edge_or_chain() {
    // Arrange
    let edge = edge();
    let chain = chain();

    // Act
    let edge_source =
        ParticleGroupSource::stroke_shape(edge).expect("edge should form a stroke source");
    let chain_source =
        ParticleGroupSource::stroke_shape(chain).expect("chain should form a stroke source");

    // Assert
    assert!(matches!(edge_source, ParticleGroupSource::StrokeShape(_)));
    assert!(matches!(chain_source, ParticleGroupSource::StrokeShape(_)));
}

#[test]
fn positions_source_owns_non_empty_source_order() {
    // Arrange
    let positions = vec![Vec2::new(2.0, 1.0), Vec2::new(-3.0, 4.0)];

    // Act
    let source = ParticleGroupSource::positions(positions.clone())
        .expect("finite positions should form a source");

    // Assert
    let ParticleGroupSource::Positions(explicit) = source else {
        panic!("positions constructor should select the positions variant");
    };
    assert_eq!(explicit.positions(), positions);
}

#[test]
fn destination_remains_independent_from_source() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key should remain available");
    let group = ParticleGroupId::from_identity(Identity::new(world, 2, 3));

    // Act
    let recipe = ParticleGroupRecipe::new(
        positions_source(),
        ParticleGroupDestination::AppendTo(group),
    );

    // Assert
    assert_eq!(
        recipe.destination(),
        ParticleGroupDestination::AppendTo(group)
    );
    assert!(matches!(recipe.source(), ParticleGroupSource::Positions(_)));
}

#[test]
fn public_group_flags_name_only_the_three_pinned_bits() {
    // Arrange
    let public =
        ParticleGroupFlags::SOLID | ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY;
    let unknown = ParticleGroupFlags::from_bits_retain(0x8000_0000);
    let private = ParticleGroupFlags::from_bits_retain(0x0018);

    // Act
    let named_bits = ParticleGroupFlags::all().bits();

    // Assert
    assert_eq!(ParticleGroupFlags::SOLID.bits(), 0x0001);
    assert_eq!(ParticleGroupFlags::RIGID.bits(), 0x0002);
    assert_eq!(ParticleGroupFlags::CAN_BE_EMPTY.bits(), 0x0004);
    assert_eq!(named_bits, public.bits());
    assert_eq!(unknown.bits(), 0x8000_0000);
    assert!(private.is_empty());
}

#[test]
fn empty_sources_are_rejected() {
    // Arrange
    let shapes = Vec::new();
    let positions = Vec::new();

    // Act
    let filled_error =
        ParticleGroupSource::filled_shapes(shapes).expect_err("empty fill must fail");
    let positions_error =
        ParticleGroupSource::positions(positions).expect_err("empty positions must fail");

    // Assert
    assert_eq!(filled_error, ParticleGroupRecipeError::EmptySource);
    assert_eq!(positions_error, ParticleGroupRecipeError::EmptySource);
}

#[test]
fn unsupported_sampling_shapes_are_rejected() {
    // Arrange
    let unfillable = edge();
    let unstrokable = circle();

    // Act
    let filled_error = ParticleGroupSource::filled_shapes(vec![unfillable])
        .expect_err("edge has no fillable interior");
    let stroke_error =
        ParticleGroupSource::stroke_shape(unstrokable).expect_err("circle is not a stroke shape");

    // Assert
    assert_eq!(
        filled_error,
        ParticleGroupRecipeError::UnsupportedFilledShape
    );
    assert_eq!(
        stroke_error,
        ParticleGroupRecipeError::UnsupportedStrokeShape
    );
}

#[test]
fn non_finite_explicit_positions_are_rejected() {
    // Arrange
    let positions = vec![Vec2::ZERO, Vec2::new(f32::NAN, 0.0)];

    // Act
    let error =
        ParticleGroupSource::positions(positions).expect_err("non-finite position must fail");

    // Assert
    assert_eq!(
        error,
        ParticleGroupRecipeError::NonFinitePosition { index: 1 }
    );
}

#[test]
fn recipe_accepts_every_checked_creation_property() {
    // Arrange
    let transform = Transform::from_position_angle(Vec2::new(2.0, -1.0), 0.25);
    let color = ParticleColor::new(1, 2, 3, 4);

    // Act
    let recipe = recipe()
        .with_particle_flags(ParticleFlags::ELASTIC)
        .with_group_flags(ParticleGroupFlags::RIGID)
        .with_transform(transform)
        .expect("finite transform should be valid")
        .with_linear_velocity(Vec2::new(3.0, 4.0))
        .expect("finite velocity should be valid")
        .with_angular_velocity(-2.0)
        .expect("finite angular velocity should be valid")
        .with_color(color)
        .with_strength(0.5)
        .expect("non-negative strength should be valid")
        .with_stride(0.125)
        .expect("positive stride should be valid")
        .with_lifetime(8.0)
        .expect("finite lifetime should be valid")
        .with_user_association("group");

    // Assert
    assert_eq!(recipe.particle_flags(), ParticleFlags::ELASTIC);
    assert_eq!(recipe.group_flags(), ParticleGroupFlags::RIGID);
    assert_eq!(recipe.transform(), transform);
    assert_eq!(recipe.linear_velocity(), Vec2::new(3.0, 4.0));
    assert_eq!(recipe.angular_velocity().to_bits(), (-2.0_f32).to_bits());
    assert_eq!(recipe.color(), color);
    assert_eq!(recipe.strength().to_bits(), 0.5_f32.to_bits());
    assert_eq!(recipe.maybe_stride(), Some(0.125));
    assert_eq!(recipe.lifetime().to_bits(), 8.0_f32.to_bits());
    assert_eq!(recipe.maybe_user_association(), Some(&"group"));
}

#[test]
fn non_finite_motion_and_lifetime_are_rejected() {
    // Arrange
    let invalid_transform = Transform::new(Vec2::ZERO, Rotation::from_angle(f32::NAN));

    // Act
    let transform_error = recipe()
        .with_transform(invalid_transform)
        .expect_err("non-finite transform must fail");
    let linear_error = recipe()
        .with_linear_velocity(Vec2::new(0.0, f32::INFINITY))
        .expect_err("non-finite linear velocity must fail");
    let angular_error = recipe()
        .with_angular_velocity(f32::NEG_INFINITY)
        .expect_err("non-finite angular velocity must fail");
    let lifetime_error = recipe()
        .with_lifetime(f32::NAN)
        .expect_err("non-finite lifetime must fail");

    // Assert
    assert_eq!(
        transform_error,
        ParticleGroupRecipeError::NonFiniteTransform
    );
    assert_eq!(
        linear_error,
        ParticleGroupRecipeError::NonFiniteLinearVelocity
    );
    assert_eq!(
        angular_error,
        ParticleGroupRecipeError::NonFiniteAngularVelocity
    );
    assert_eq!(lifetime_error, ParticleGroupRecipeError::NonFiniteLifetime);
}

#[test]
fn invalid_strength_and_stride_are_rejected() {
    // Arrange
    let non_finite_strength = f32::INFINITY;
    let negative_strength = -0.25;
    let non_finite_stride = f32::NAN;
    let zero_stride = 0.0;

    // Act
    let strength_finite_error = recipe()
        .with_strength(non_finite_strength)
        .expect_err("non-finite strength must fail");
    let strength_sign_error = recipe()
        .with_strength(negative_strength)
        .expect_err("negative strength must fail");
    let stride_finite_error = recipe()
        .with_stride(non_finite_stride)
        .expect_err("non-finite stride must fail");
    let stride_sign_error = recipe()
        .with_stride(zero_stride)
        .expect_err("zero stride must fail");

    // Assert
    assert_eq!(
        strength_finite_error,
        ParticleGroupRecipeError::NonFiniteStrength
    );
    assert_eq!(
        strength_sign_error,
        ParticleGroupRecipeError::NegativeStrength
    );
    assert_eq!(
        stride_finite_error,
        ParticleGroupRecipeError::NonFiniteStride
    );
    assert_eq!(
        stride_sign_error,
        ParticleGroupRecipeError::NonPositiveStride
    );
}

#[test]
fn group_view_exposes_complete_stable_read_only_state() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key should remain available");
    let members = [particle_id(world, 1), particle_id(world, 2)];
    let depths = [0.25, 0.5];
    let state = view_state(world);

    // Act
    let view = ParticleGroupView::new(state, &members, Some(&depths))
        .expect("aligned group state should form a view");

    // Assert
    assert_eq!(view.id(), state.id);
    assert_eq!(view.flags(), state.flags);
    assert_eq!(view.transform(), state.transform);
    assert_eq!(view.position(), Vec2::new(3.0, 4.0));
    assert_eq!(view.angle().to_bits(), 0.25_f32.to_bits());
    assert_eq!(view.center(), state.center);
    assert_eq!(view.linear_velocity(), state.linear_velocity);
    assert_eq!(
        view.angular_velocity().to_bits(),
        state.angular_velocity.to_bits()
    );
    assert_eq!(view.mass().to_bits(), state.mass.to_bits());
    assert_eq!(view.inertia().to_bits(), state.inertia.to_bits());
    assert_eq!(view.member_count(), 2);
    assert_eq!(view.member_ids(), members);
    assert_eq!(view.maybe_depths(), Some(depths.as_slice()));
}

#[test]
fn group_view_rejects_depth_not_aligned_with_members() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key should remain available");
    let members = [particle_id(world, 1), particle_id(world, 2)];
    let depths = [0.25];

    // Act
    let error = ParticleGroupView::new(view_state(world), &members, Some(&depths))
        .expect_err("misaligned depth must not form a view");

    // Assert
    assert_eq!(
        error,
        ParticleGroupViewError::MisalignedDepth {
            member_count: 2,
            depth_count: 1,
        }
    );
}

#[test]
fn retained_empty_group_reports_exact_zero_aggregate_state() {
    // Arrange
    let world = WorldKey::fresh().expect("test world key should remain available");
    let members = [];
    let depths = [];
    let state = view_state(world);

    // Act
    let view = ParticleGroupView::new(state, &members, Some(&depths))
        .expect("aligned empty group should form a view");

    // Assert
    assert_eq!(view.member_count(), 0);
    assert!(view.member_ids().is_empty());
    assert_eq!(view.maybe_depths(), Some(depths.as_slice()));
    assert_eq!(view.center(), Vec2::ZERO);
    assert_eq!(view.linear_velocity(), Vec2::ZERO);
    assert_eq!(view.angular_velocity().to_bits(), 0.0_f32.to_bits());
    assert_eq!(view.mass().to_bits(), 0.0_f32.to_bits());
    assert_eq!(view.inertia().to_bits(), 0.0_f32.to_bits());
    assert_eq!(view.transform(), state.transform);
}

#[test]
fn member_and_depth_borrows_share_the_view_storage_lifetime() {
    fn borrow_members<'view>(view: &'view ParticleGroupView<'view>) -> &'view [ParticleId] {
        view.member_ids()
    }

    fn borrow_depths<'view>(view: &'view ParticleGroupView<'view>) -> Option<&'view [f32]> {
        view.maybe_depths()
    }

    // Arrange
    let world = WorldKey::fresh().expect("test world key should remain available");
    let members = [particle_id(world, 1)];
    let depths = [0.75];
    let view = ParticleGroupView::new(view_state(world), &members, Some(&depths))
        .expect("aligned group state should form a view");

    // Act
    let borrowed_members = borrow_members(&view);
    let borrowed_depths = borrow_depths(&view);

    // Assert
    assert_eq!(borrowed_members, members);
    assert_eq!(borrowed_depths, Some(depths.as_slice()));
}
