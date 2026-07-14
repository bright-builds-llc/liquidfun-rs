//! Bounded, renderer-neutral semantic diagnostics for rigid differential evidence.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;

use crate::math::Vec2;
use crate::{BodyId, BodySnapshot, FixtureSnapshot, JointDef, JointId, JointKind, JointSnapshot};

use super::object::World;

const REVIEWED_MAX_RECONSTRUCTION_BODIES: usize = 4_096;
const REVIEWED_MAX_RECONSTRUCTION_FIXTURES: usize = 8_192;
const REVIEWED_MAX_RECONSTRUCTION_JOINTS: usize = 8_192;

/// Reviewed finite capacities for one reconstruction snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldReconstructionLimits {
    bodies: usize,
    fixtures: usize,
    joints: usize,
}

impl WorldReconstructionLimits {
    /// Returns the capacities applied to every reconstruction.
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            bodies: REVIEWED_MAX_RECONSTRUCTION_BODIES,
            fixtures: REVIEWED_MAX_RECONSTRUCTION_FIXTURES,
            joints: REVIEWED_MAX_RECONSTRUCTION_JOINTS,
        }
    }

    /// Returns the live-body capacity.
    #[must_use]
    pub const fn max_bodies(self) -> usize {
        self.bodies
    }

    /// Returns the live-fixture capacity.
    #[must_use]
    pub const fn max_fixtures(self) -> usize {
        self.fixtures
    }

    /// Returns the live-joint capacity.
    #[must_use]
    pub const fn max_joints(self) -> usize {
        self.joints
    }
}

/// A checked, output-local coordinate in one semantic reconstruction.
///
/// Indices describe only the owned record graph. They are not arena slots,
/// handles, or reusable world identities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReconstructionIndex(u32);

impl ReconstructionIndex {
    fn checked(value: usize) -> Result<Self, WorldReconstructionError> {
        let value =
            u32::try_from(value).map_err(|_error| WorldReconstructionError::InvalidState {
                resource: "reconstruction index",
            })?;
        Ok(Self(value))
    }

    /// Returns the zero-based output-local coordinate.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A pinned upstream limitation that prevents faithful reconstruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconstructionUnsupported {
    /// The selected upstream mouse joint deliberately has no reconstructable dump.
    MouseJoint,
}

/// Whether the pinned source exposes enough state to reconstruct one record.
#[derive(Debug, Clone, PartialEq)]
pub enum ReconstructionSupport<T> {
    /// Complete typed semantic state is present.
    Supported(T),
    /// The pinned source explicitly does not support this reconstruction.
    Unsupported(ReconstructionUnsupported),
}

/// One fixture nested under its owning body record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureReconstruction {
    index: ReconstructionIndex,
    snapshot: FixtureSnapshot,
}

impl FixtureReconstruction {
    /// Returns the fixture's output-local coordinate.
    #[must_use]
    pub const fn index(&self) -> ReconstructionIndex {
        self.index
    }

    /// Returns the complete owned shape, material, sensor, and filter state.
    #[must_use]
    pub const fn snapshot(&self) -> &FixtureSnapshot {
        &self.snapshot
    }
}

/// One body and its source-ordered fixture records.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyReconstruction {
    index: ReconstructionIndex,
    snapshot: BodySnapshot,
    fixtures: Vec<FixtureReconstruction>,
}

impl BodyReconstruction {
    /// Returns the body's output-local coordinate.
    #[must_use]
    pub const fn index(&self) -> ReconstructionIndex {
        self.index
    }

    /// Returns the complete owned semantic body state.
    #[must_use]
    pub const fn snapshot(&self) -> BodySnapshot {
        self.snapshot
    }

    /// Returns fixtures in the owning body's newest-first source order.
    #[must_use]
    pub fn fixtures(&self) -> &[FixtureReconstruction] {
        &self.fixtures
    }
}

/// One joint record with output-local body and gear-dependency coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct JointReconstruction {
    index: ReconstructionIndex,
    kind: JointKind,
    body_indices: [ReconstructionIndex; 2],
    collide_connected: bool,
    anchor_a: Vec2,
    anchor_b: Vec2,
    maybe_source_joint_indices: Option<[ReconstructionIndex; 2]>,
    support: ReconstructionSupport<JointSnapshot>,
}

impl JointReconstruction {
    /// Returns the joint's coordinate from the original newest-first joint pass.
    #[must_use]
    pub const fn index(&self) -> ReconstructionIndex {
        self.index
    }

    /// Returns the closed concrete joint kind.
    #[must_use]
    pub const fn kind(&self) -> JointKind {
        self.kind
    }

    /// Returns the two output-local body coordinates.
    #[must_use]
    pub const fn body_indices(&self) -> [ReconstructionIndex; 2] {
        self.body_indices
    }

    /// Returns whether the connected bodies may collide.
    #[must_use]
    pub const fn collide_connected(&self) -> bool {
        self.collide_connected
    }

    /// Returns the current world-space anchor on body A.
    #[must_use]
    pub const fn anchor_a(&self) -> Vec2 {
        self.anchor_a
    }

    /// Returns the current world-space anchor on body B.
    #[must_use]
    pub const fn anchor_b(&self) -> Vec2 {
        self.anchor_b
    }

    /// Returns source-joint coordinates for a gear joint, or no dependencies otherwise.
    #[must_use]
    pub const fn maybe_source_joint_indices(&self) -> Option<[ReconstructionIndex; 2]> {
        self.maybe_source_joint_indices
    }

    /// Returns complete typed state or the pinned explicit unsupported classification.
    #[must_use]
    pub const fn support(&self) -> &ReconstructionSupport<JointSnapshot> {
        &self.support
    }
}

/// An owned, bounded semantic reconstruction of rigid world state.
///
/// This is diagnostic evidence, not a persistence format or round-trip API.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldReconstruction {
    gravity: Vec2,
    bodies: Vec<BodyReconstruction>,
    joints: Vec<JointReconstruction>,
}

impl WorldReconstruction {
    /// Returns the captured world gravity.
    #[must_use]
    pub const fn gravity(&self) -> Vec2 {
        self.gravity
    }

    /// Returns bodies with their fixtures before every joint record.
    #[must_use]
    pub fn bodies(&self) -> &[BodyReconstruction] {
        &self.bodies
    }

    /// Returns non-gear joints first and gear joints last.
    #[must_use]
    pub fn joints(&self) -> &[JointReconstruction] {
        &self.joints
    }

    /// Renders a deterministic one-way human diagnostic view.
    ///
    /// The text is intentionally incomplete and has no parsing, whitespace,
    /// locale, persistence, or round-trip compatibility contract.
    #[must_use]
    pub fn to_diagnostic_text(&self) -> String {
        let mut text = String::new();
        writeln!(
            text,
            "world-reconstruction diagnostic-only; not persistence or round-trip data"
        )
        .expect("writing to an owned String cannot fail");
        writeln!(
            text,
            "gravity x_bits={:08x} y_bits={:08x}",
            self.gravity.x.to_bits(),
            self.gravity.y.to_bits()
        )
        .expect("writing to an owned String cannot fail");
        for body in &self.bodies {
            let snapshot = body.snapshot;
            writeln!(
                text,
                "body[{}] kind={:?} position_bits=[{:08x},{:08x}] angle_bits={:08x} fixtures={}",
                body.index.get(),
                snapshot.body_type(),
                snapshot.position().x.to_bits(),
                snapshot.position().y.to_bits(),
                snapshot.angle().to_bits(),
                body.fixtures.len()
            )
            .expect("writing to an owned String cannot fail");
            for fixture in &body.fixtures {
                writeln!(
                    text,
                    "  fixture[{}] shape={:?} density_bits={:08x} friction_bits={:08x} restitution_bits={:08x} sensor={} filter={:?}",
                    fixture.index.get(),
                    fixture.snapshot.shape(),
                    fixture.snapshot.density().to_bits(),
                    fixture.snapshot.friction().to_bits(),
                    fixture.snapshot.restitution().to_bits(),
                    fixture.snapshot.is_sensor(),
                    fixture.snapshot.filter_data()
                )
                .expect("writing to an owned String cannot fail");
            }
        }
        for joint in &self.joints {
            let support = match joint.support {
                ReconstructionSupport::Supported(_) => "supported",
                ReconstructionSupport::Unsupported(ReconstructionUnsupported::MouseJoint) => {
                    "unsupported(mouse-joint)"
                }
            };
            writeln!(
                text,
                "joint[{}] kind={:?} bodies=[{},{}] collide_connected={} support={support}",
                joint.index.get(),
                joint.kind,
                joint.body_indices[0].get(),
                joint.body_indices[1].get(),
                joint.collide_connected
            )
            .expect("writing to an owned String cannot fail");
        }
        text
    }
}

/// A bounded failure while collecting one semantic reconstruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldReconstructionError {
    /// A reviewed record collection exceeded its finite capacity.
    CapacityExceeded {
        /// Stable semantic resource name.
        resource: &'static str,
        /// Reviewed finite limit.
        limit: usize,
    },
    /// Live private graph state did not resolve to a complete semantic graph.
    InvalidState {
        /// Stable semantic resource name.
        resource: &'static str,
    },
}

impl fmt::Display for WorldReconstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded { resource, limit } => {
                write!(
                    formatter,
                    "{resource} exceeds reviewed reconstruction limit {limit}"
                )
            }
            Self::InvalidState { resource } => {
                write!(
                    formatter,
                    "world reconstruction has invalid {resource} state"
                )
            }
        }
    }
}

impl Error for WorldReconstructionError {}

/// Exact renderer-neutral world counts and dynamic-tree metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldDiagnostics {
    body_count: usize,
    fixture_count: usize,
    joint_count: usize,
    contact_count: usize,
    manifold_point_count: usize,
    proxy_count: usize,
    tree_height: i32,
    tree_balance: i32,
    tree_quality: f32,
}

impl WorldDiagnostics {
    /// Returns the exact live body count.
    #[must_use]
    pub const fn body_count(self) -> usize {
        self.body_count
    }

    /// Returns the exact live fixture count.
    #[must_use]
    pub const fn fixture_count(self) -> usize {
        self.fixture_count
    }

    /// Returns the exact live joint count.
    #[must_use]
    pub const fn joint_count(self) -> usize {
        self.joint_count
    }

    /// Returns the exact private contact-occurrence count.
    #[must_use]
    pub const fn contact_count(self) -> usize {
        self.contact_count
    }

    /// Returns the exact total number of current manifold points.
    #[must_use]
    pub const fn manifold_point_count(self) -> usize {
        self.manifold_point_count
    }

    /// Returns the exact broad-phase proxy count.
    #[must_use]
    pub const fn proxy_count(self) -> usize {
        self.proxy_count
    }

    /// Returns the exact dynamic-tree root height.
    #[must_use]
    pub const fn tree_height(self) -> i32 {
        self.tree_height
    }

    /// Returns the exact maximum dynamic-tree child-height difference.
    #[must_use]
    pub const fn tree_balance(self) -> i32 {
        self.tree_balance
    }

    /// Returns the exact total-to-root dynamic-tree perimeter ratio.
    ///
    /// Later differential policy names how this floating observation compares.
    #[must_use]
    pub const fn tree_quality(self) -> f32 {
        self.tree_quality
    }
}

impl World {
    /// Copies a bounded typed semantic reconstruction.
    ///
    /// # Errors
    ///
    /// Returns a typed error when reviewed record bounds are exceeded or the
    /// live semantic graph cannot be resolved without storage coordinates.
    #[doc(hidden)]
    pub fn semantic_reconstruction(&self) -> Result<WorldReconstruction, WorldReconstructionError> {
        let limits = WorldReconstructionLimits::reviewed();
        let mut ordered_bodies = self.bodies.iter().collect::<Vec<_>>();
        ordered_bodies.sort_by_key(|(_id, body)| Reverse(body.diagnostic_id));
        check_bound("bodies", ordered_bodies.len(), limits.max_bodies())?;
        let fixture_count = ordered_bodies
            .iter()
            .map(|(_id, body)| body.fixtures.len())
            .sum();
        check_bound("fixtures", fixture_count, limits.max_fixtures())?;

        let mut body_indices = HashMap::with_capacity(ordered_bodies.len());
        let mut bodies = Vec::with_capacity(ordered_bodies.len());
        let mut next_fixture_index = 0_usize;
        for (body_position, (body_id, body)) in ordered_bodies.into_iter().enumerate() {
            let index = ReconstructionIndex::checked(body_position)?;
            body_indices.insert(body_id, index);
            let mut fixtures = Vec::with_capacity(body.fixtures.len());
            for fixture_id in &body.fixtures {
                let fixture = self.fixtures.get(*fixture_id).map_err(|_error| {
                    WorldReconstructionError::InvalidState {
                        resource: "fixture ownership",
                    }
                })?;
                fixtures.push(FixtureReconstruction {
                    index: ReconstructionIndex::checked(next_fixture_index)?,
                    snapshot: fixture.definition.snapshot(),
                });
                next_fixture_index += 1;
            }
            bodies.push(BodyReconstruction {
                index,
                snapshot: body.state.snapshot(),
                fixtures,
            });
        }

        let mut ordered_joints = self.joints.iter().collect::<Vec<_>>();
        ordered_joints.sort_by_key(|(_id, joint)| Reverse(joint.diagnostic_id));
        check_bound("joints", ordered_joints.len(), limits.max_joints())?;
        let mut joint_indices = HashMap::with_capacity(ordered_joints.len());
        for (joint_position, (joint_id, _joint)) in ordered_joints.iter().enumerate() {
            joint_indices.insert(*joint_id, ReconstructionIndex::checked(joint_position)?);
        }

        let mut joints = Vec::with_capacity(ordered_joints.len());
        for gear_pass in [false, true] {
            for (joint_id, record) in &ordered_joints {
                let is_gear = matches!(record.definition, JointDef::Gear(_));
                if is_gear != gear_pass {
                    continue;
                }
                joints.push(self.reconstruct_joint(
                    *joint_id,
                    record.definition,
                    &body_indices,
                    &joint_indices,
                )?);
            }
        }

        Ok(WorldReconstruction {
            gravity: self.gravity(),
            bodies,
            joints,
        })
    }

    fn reconstruct_joint(
        &self,
        joint_id: JointId,
        definition: JointDef,
        body_indices: &HashMap<BodyId, ReconstructionIndex>,
        joint_indices: &HashMap<JointId, ReconstructionIndex>,
    ) -> Result<JointReconstruction, WorldReconstructionError> {
        let snapshot = self.joint_snapshot(joint_id).map_err(|_error| {
            WorldReconstructionError::InvalidState {
                resource: "joint snapshot",
            }
        })?;
        let [body_a, body_b] = snapshot.bodies();
        let body_indices = [
            mapped_index(body_indices, body_a, "joint body A")?,
            mapped_index(body_indices, body_b, "joint body B")?,
        ];
        let maybe_source_joint_indices = match definition {
            JointDef::Gear(gear) => {
                let [source_a, source_b] = gear.source_joints();
                Some([
                    mapped_index(joint_indices, source_a, "gear source A")?,
                    mapped_index(joint_indices, source_b, "gear source B")?,
                ])
            }
            _ => None,
        };
        let support = if snapshot.kind() == JointKind::Mouse {
            ReconstructionSupport::Unsupported(ReconstructionUnsupported::MouseJoint)
        } else {
            ReconstructionSupport::Supported(snapshot)
        };
        Ok(JointReconstruction {
            index: mapped_index(joint_indices, joint_id, "joint index")?,
            kind: snapshot.kind(),
            body_indices,
            collide_connected: snapshot.collide_connected(),
            anchor_a: snapshot.anchor_a(),
            anchor_b: snapshot.anchor_b(),
            maybe_source_joint_indices,
            support,
        })
    }

    /// Copies exact Phase 8 counts and dynamic-tree metrics.
    #[doc(hidden)]
    #[must_use]
    pub fn world_diagnostics(&self) -> WorldDiagnostics {
        let manifold_point_count = self
            .contact_manager
            .contacts()
            .iter()
            .map(|contact| contact.snapshot().points().len())
            .sum();
        WorldDiagnostics {
            body_count: self.bodies.iter().count(),
            fixture_count: self.fixtures.iter().count(),
            joint_count: self.joints.iter().count(),
            contact_count: self.contact_manager.len(),
            manifold_point_count,
            proxy_count: self.broad_phase.proxy_count(),
            tree_height: self.broad_phase.tree_height(),
            tree_balance: self.broad_phase.tree_max_balance(),
            tree_quality: self.broad_phase.tree_area_ratio(),
        }
    }
}

fn check_bound(
    resource: &'static str,
    count: usize,
    limit: usize,
) -> Result<(), WorldReconstructionError> {
    if count > limit {
        return Err(WorldReconstructionError::CapacityExceeded { resource, limit });
    }
    Ok(())
}

fn mapped_index<Id: Copy + Eq + std::hash::Hash>(
    indices: &HashMap<Id, ReconstructionIndex>,
    id: Id,
    resource: &'static str,
) -> Result<ReconstructionIndex, WorldReconstructionError> {
    indices
        .get(&id)
        .copied()
        .ok_or(WorldReconstructionError::InvalidState { resource })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewed_reconstruction_bound_rejects_first_excess_record() {
        // Arrange
        let limits = WorldReconstructionLimits::reviewed();

        // Act
        let at_limit = check_bound("bodies", limits.max_bodies(), limits.max_bodies());
        let excess = check_bound("bodies", limits.max_bodies() + 1, limits.max_bodies());

        // Assert
        assert_eq!(at_limit, Ok(()));
        assert_eq!(
            excess,
            Err(WorldReconstructionError::CapacityExceeded {
                resource: "bodies",
                limit: limits.max_bodies(),
            })
        );
    }
}
