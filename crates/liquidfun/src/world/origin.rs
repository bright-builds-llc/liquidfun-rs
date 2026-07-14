//! Checked world-origin translation over rigid body and broad-phase state.

use std::error::Error;
use std::fmt;

use crate::collision::TreeError;
use crate::collision::broad_phase::PreparedBroadPhaseOriginShift;
use crate::math::Vec2;
use crate::{BodyId, FixtureId, JointDef, JointId, World};

use super::body::BodyState;
use super::proxy::{PreparedProxyOriginShift, ProxyOriginShiftError};

struct OriginShiftCandidate {
    body_states: Vec<(BodyId, BodyState)>,
    joint_definitions: Vec<(JointId, JointDef)>,
    proxy_states: Vec<(FixtureId, PreparedProxyOriginShift)>,
    broad_phase: PreparedBroadPhaseOriginShift,
}

/// A failure while translating every rigid world-space coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OriginShiftError {
    /// A previous hook panic poisoned coherent world operations.
    Poisoned,
    /// The world is inside an active step.
    Locked,
    /// At least one requested shift coordinate is NaN or infinite.
    NonFiniteShift,
    /// Translating a body transform or sweep produced a non-finite coordinate.
    NonFiniteBodyState,
    /// Translating a fixture or tree bound produced a non-finite coordinate.
    NonFiniteProxyBounds,
    /// Translating a joint world anchor or target produced a non-finite coordinate.
    NonFiniteJointState,
    /// Fixture proxy bookkeeping did not match live broad-phase storage.
    InconsistentProxy,
    /// Private broad-phase state rejected an otherwise checked translation.
    InternalBroadPhaseState,
}

impl fmt::Display for OriginShiftError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Poisoned => "world is poisoned by a previous hook panic",
            Self::Locked => "world is locked by an active step",
            Self::NonFiniteShift => "world origin shift must be finite",
            Self::NonFiniteBodyState => {
                "world origin shift produced a non-finite body transform or sweep"
            }
            Self::NonFiniteProxyBounds => {
                "world origin shift produced non-finite broad-phase bounds"
            }
            Self::NonFiniteJointState => {
                "world origin shift produced a non-finite joint anchor or target"
            }
            Self::InconsistentProxy => {
                "world fixture proxy bookkeeping is inconsistent with the broad phase"
            }
            Self::InternalBroadPhaseState => {
                "world broad-phase invariant failed during origin shifting"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for OriginShiftError {}

impl World {
    /// Subtracts `shift` from every rigid world-space coordinate atomically.
    ///
    /// Body and fixture identities, local geometry, velocity, forces, contacts,
    /// filtering, broad-phase topology, and buffered moves are preserved. Pulley
    /// ground anchors and mouse targets translate with the rigid world.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error when the world is poisoned or locked,
    /// the shift is non-finite, any finite subtraction overflows, or fixture
    /// proxy bookkeeping is inconsistent with the broad phase.
    pub fn shift_origin(&mut self, shift: Vec2) -> Result<(), OriginShiftError> {
        let candidate = self.prepare_origin_shift(shift)?;
        self.commit_origin_shift(candidate);
        Ok(())
    }

    fn prepare_origin_shift(&self, shift: Vec2) -> Result<OriginShiftCandidate, OriginShiftError> {
        if self.step_state.is_poisoned() {
            return Err(OriginShiftError::Poisoned);
        }
        if self.step_state.is_locked() {
            return Err(OriginShiftError::Locked);
        }
        if !shift.is_valid() {
            return Err(OriginShiftError::NonFiniteShift);
        }

        let body_states = self
            .bodies
            .iter()
            .map(|(body, record)| {
                let Some(state) = record.state.maybe_shifted_origin(shift) else {
                    return Err(OriginShiftError::NonFiniteBodyState);
                };
                Ok((body, state))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let joint_definitions = self
            .joints
            .iter()
            .map(|(joint, record)| {
                record
                    .shifted_definition(shift)
                    .map(|definition| (joint, definition))
                    .map_err(|_error| OriginShiftError::NonFiniteJointState)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut proxy_count = 0_usize;
        let proxy_states = self
            .fixtures
            .iter()
            .map(|(fixture, record)| {
                let body = self
                    .bodies
                    .get(record.body)
                    .map_err(|_error| OriginShiftError::InconsistentProxy)?;
                let prepared = record
                    .proxies
                    .prepare_origin_shift(
                        &self.broad_phase,
                        fixture,
                        record.body,
                        record.definition.shape(),
                        body.state.snapshot().is_active(),
                        shift,
                    )
                    .map_err(map_proxy_error)?;
                proxy_count += record.proxies.len();
                Ok((fixture, prepared))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if proxy_count != self.broad_phase.proxy_count() {
            return Err(OriginShiftError::InconsistentProxy);
        }

        let broad_phase = self
            .broad_phase
            .prepare_origin_shift(shift)
            .map_err(map_tree_error)?;
        Ok(OriginShiftCandidate {
            body_states,
            joint_definitions,
            proxy_states,
            broad_phase,
        })
    }

    fn commit_origin_shift(&mut self, candidate: OriginShiftCandidate) {
        for (body, state) in candidate.body_states {
            self.bodies
                .get_mut(body)
                .expect("prepared origin-shift body remains live during commit")
                .state = state;
        }
        for (joint, definition) in candidate.joint_definitions {
            self.joints
                .get_mut(joint)
                .expect("prepared origin-shift joint remains live during commit")
                .definition = definition;
        }
        for (fixture, prepared) in candidate.proxy_states {
            self.fixtures
                .get_mut(fixture)
                .expect("prepared origin-shift fixture remains live during commit")
                .proxies
                .commit_origin_shift(prepared);
        }
        self.broad_phase.commit_origin_shift(candidate.broad_phase);
    }
}

fn map_proxy_error(error: ProxyOriginShiftError) -> OriginShiftError {
    match error {
        ProxyOriginShiftError::InconsistentProxy => OriginShiftError::InconsistentProxy,
        ProxyOriginShiftError::NonFiniteBounds => OriginShiftError::NonFiniteProxyBounds,
    }
}

fn map_tree_error(error: TreeError) -> OriginShiftError {
    match error {
        TreeError::NonFiniteOriginShift => OriginShiftError::NonFiniteShift,
        TreeError::AabbOverflow => OriginShiftError::NonFiniteProxyBounds,
        _ => OriginShiftError::InternalBroadPhaseState,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::collision::shape::{CircleShape, Shape};
    use crate::collision::{Aabb, FilterData};
    use crate::{BodyDef, BodyType, FixtureDef, WakePolicy};

    fn dynamic_body(position: Vec2) -> BodyDef {
        BodyDef::new(BodyType::Dynamic, position, 0.25, true)
            .expect("test body definition should be valid")
    }

    fn circle_fixture() -> FixtureDef {
        let shape = Shape::from(
            CircleShape::new(Vec2::new(0.25, -0.5), 0.75).expect("test circle should be valid"),
        );
        FixtureDef::new(
            shape,
            1.0,
            0.3,
            0.1,
            false,
            FilterData::new(0x0002, 0x0004, -1),
        )
        .expect("test fixture definition should be valid")
    }

    fn assert_body_state_shifted(state_before: BodyState, state_after: BodyState, shift: Vec2) {
        assert_eq!(
            state_after.snapshot().position(),
            state_before.snapshot().position() - shift
        );
        assert_eq!(
            state_after.snapshot().angle().to_bits(),
            state_before.snapshot().angle().to_bits()
        );
        assert_eq!(
            state_after.snapshot().linear_velocity(),
            state_before.snapshot().linear_velocity()
        );
        assert_eq!(
            state_after.snapshot().local_center(),
            state_before.snapshot().local_center()
        );
        assert_eq!(
            state_after.accumulated_force(),
            state_before.accumulated_force()
        );
        assert_eq!(
            state_after.accumulated_torque().to_bits(),
            state_before.accumulated_torque().to_bits()
        );
        assert_eq!(
            state_after.sweep().initial_center(),
            state_before.sweep().initial_center() - shift
        );
        assert_eq!(
            state_after.sweep().center(),
            state_before.sweep().center() - shift
        );
    }

    #[test]
    fn locked_origin_shift_is_rejected_without_mutation() {
        // Arrange
        let mut world = World::new().expect("test world key should remain available");
        let body = world
            .create_body(&dynamic_body(Vec2::new(3.0, -2.0)))
            .expect("test body should fit");
        let before = world.body_snapshot(body).expect("body should remain live");
        world.step_state.set_locked_for_test(true);

        // Act
        let result = world.shift_origin(Vec2::new(1.0, 2.0));
        world.step_state.set_locked_for_test(false);

        // Assert
        assert_eq!(result, Err(OriginShiftError::Locked));
        assert_eq!(world.body_snapshot(body), Ok(before));
    }

    #[test]
    fn inconsistent_proxy_origin_shift_is_rejected_without_further_mutation() {
        // Arrange
        let mut world = World::new().expect("test world key should remain available");
        let body = world
            .create_body(&dynamic_body(Vec2::ZERO))
            .expect("test body should fit");
        let fixture = world
            .create_fixture(body, &circle_fixture())
            .expect("test fixture should fit");
        let child = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .definition
            .shape()
            .child_index(0)
            .expect("circle child should exist");
        let proxy = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .proxies
            .maybe_proxy_id(child)
            .expect("active fixture should own a proxy");
        world
            .broad_phase
            .destroy_proxy(proxy)
            .expect("test proxy should remain live before corruption");
        let body_before = world.body_snapshot(body).expect("body should remain live");
        let move_buffer_before = world.broad_phase.move_buffer_for_origin_test().to_vec();

        // Act
        let result = world.shift_origin(Vec2::new(1.0, 2.0));

        // Assert
        assert_eq!(result, Err(OriginShiftError::InconsistentProxy));
        assert_eq!(world.body_snapshot(body), Ok(body_before));
        assert_eq!(
            world.broad_phase.move_buffer_for_origin_test(),
            move_buffer_before
        );
    }

    #[test]
    fn successful_origin_shift_preserves_proxy_identity_and_rigid_state() {
        // Arrange
        let mut world = World::new().expect("test world key should remain available");
        let body = world
            .create_body(&dynamic_body(Vec2::new(6.0, -4.0)))
            .expect("test body should fit");
        let fixture = world
            .create_fixture(body, &circle_fixture())
            .expect("test fixture should fit");
        world
            .set_body_linear_velocity(body, Vec2::new(2.0, -3.0))
            .expect("finite velocity should succeed");
        world
            .apply_body_force_to_center(body, Vec2::new(5.0, 7.0), WakePolicy::Wake)
            .expect("finite force should succeed");
        let child = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .definition
            .shape()
            .child_index(0)
            .expect("circle child should exist");
        let proxy = world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .proxies
            .maybe_proxy_id(child)
            .expect("active fixture should own a proxy");
        let proxy_bounds_before = world
            .broad_phase
            .fat_aabb(proxy)
            .expect("proxy should remain live");
        let filter_before = world
            .broad_phase
            .filter_data(proxy)
            .expect("proxy should remain live");
        let move_buffer_before = world.broad_phase.move_buffer_for_origin_test().to_vec();
        let state_before = world
            .bodies
            .get(body)
            .expect("body should remain live")
            .state;
        let shift = Vec2::new(10.0, -8.0);

        // Act
        world
            .shift_origin(shift)
            .expect("finite translated state should remain valid");

        // Assert
        let state_after = world
            .bodies
            .get(body)
            .expect("body should remain live")
            .state;
        assert_body_state_shifted(state_before, state_after, shift);
        assert_eq!(
            world
                .fixtures
                .get(fixture)
                .expect("fixture should remain live")
                .proxies
                .maybe_proxy_id(child),
            Some(proxy)
        );
        assert_eq!(world.broad_phase.filter_data(proxy), Ok(filter_before));
        assert_eq!(
            world.broad_phase.fat_aabb(proxy),
            Ok(Aabb::new(
                proxy_bounds_before.lower_bound() - shift,
                proxy_bounds_before.upper_bound() - shift,
            )
            .expect("shifted proxy bounds should be valid"))
        );
        assert_eq!(
            world.broad_phase.move_buffer_for_origin_test(),
            move_buffer_before
        );
    }
}
