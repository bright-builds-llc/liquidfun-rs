#![allow(
    missing_docs,
    reason = "closed private-harness policy errors and accessors are self-describing"
)]

use std::collections::HashSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{
    CollectionPolicy, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy, FloatPolicy,
    NonFinitePolicy, ZeroPolicy,
};
use crate::{RigidWorldWitnessFamily, Sha256Hex, ToleranceProfileVersion};

const MAXIMUM_SEMANTIC_PATH_BYTES: usize = 256;
const MAXIMUM_JUSTIFICATION_BYTES: usize = 512;

const STRUCTURAL_PATHS: &[&str] = &[
    "rigid_world.result.request_id",
    "rigid_world.result.scenario_id",
    "rigid_world.timelines.order",
    "rigid_world.timeline.witness_family",
    "rigid_world.checkpoints.order",
    "rigid_world.checkpoint.id",
    "rigid_world.checkpoint.phase",
    "rigid_world.checkpoint.counts",
    "rigid_world.checkpoint.bodies.declaration_order",
    "rigid_world.body.id",
    "rigid_world.body.kind",
    "rigid_world.body.active",
    "rigid_world.checkpoint.fixtures.declaration_order",
    "rigid_world.fixture.id",
    "rigid_world.fixture.owner_body_id",
    "rigid_world.fixture.sensor",
    "rigid_world.fixture.filter.category_bits",
    "rigid_world.fixture.filter.mask_bits",
    "rigid_world.fixture.filter.group_index",
    "rigid_world.checkpoint.contacts.manager_order",
    "rigid_world.contact.identity",
    "rigid_world.contact.touching",
    "rigid_world.contact.enabled",
    "rigid_world.contact.sensor",
    "rigid_world.contact.manifold.presence",
    "rigid_world.contact.manifold.kind",
    "rigid_world.contact.manifold.points.order",
    "rigid_world.contact.manifold.point.feature",
    "rigid_world.checkpoint.events.report_order",
    "rigid_world.event.kind",
    "rigid_world.event.contact_identity",
    "rigid_world.checkpoint.destructions.report_order",
    "rigid_world.destruction.kind",
    "rigid_world.destruction.identity",
];

const FLOAT_PATHS: &[&str] = &[
    "rigid_world.body.transform.position.x",
    "rigid_world.body.transform.position.y",
    "rigid_world.body.transform.angle",
    "rigid_world.body.linear_velocity.x",
    "rigid_world.body.linear_velocity.y",
    "rigid_world.body.angular_velocity",
    "rigid_world.body.mass",
    "rigid_world.body.local_center.x",
    "rigid_world.body.local_center.y",
    "rigid_world.body.inertia",
    "rigid_world.fixture.density",
    "rigid_world.fixture.friction",
    "rigid_world.fixture.restitution",
    "rigid_world.contact.mixed_friction",
    "rigid_world.contact.mixed_restitution",
    "rigid_world.contact.manifold.local_normal.x",
    "rigid_world.contact.manifold.local_normal.y",
    "rigid_world.contact.manifold.local_point.x",
    "rigid_world.contact.manifold.local_point.y",
    "rigid_world.contact.manifold.point.position.x",
    "rigid_world.contact.manifold.point.position.y",
    "rigid_world.contact.manifold.point.normal_impulse",
    "rigid_world.contact.manifold.point.tangent_impulse",
];

const PHASE7_STRUCTURAL_PATHS: &[&str] = &[
    "rigid_world.phase7.observations.order",
    "rigid_world.phase7.body.id",
    "rigid_world.phase7.body.awake",
    "rigid_world.phase7.body.bullet",
    "rigid_world.phase7.body.sleeping_allowed",
    "rigid_world.phase7.body.fixed_rotation",
    "rigid_world.phase7.step.outcome.kind",
    "rigid_world.phase7.step.completion",
    "rigid_world.phase7.step.partial_classification",
    "rigid_world.phase7.contact.transitions.order",
    "rigid_world.phase7.contact.identity",
    "rigid_world.phase7.island.body_order",
    "rigid_world.phase7.island.contact_order",
    "rigid_world.phase7.query.completion",
    "rigid_world.phase7.query.occurrences.identity",
    "rigid_world.phase7.ray.completion",
    "rigid_world.phase7.ray.final_max_fraction",
    "rigid_world.phase7.ray.hit.identity",
];

const PHASE7_ABSOLUTE_RELATIVE_PATHS: &[&str] = &[
    "rigid_world.phase7.body.transform.position.x",
    "rigid_world.phase7.body.transform.position.y",
    "rigid_world.phase7.body.linear_velocity.x",
    "rigid_world.phase7.body.linear_velocity.y",
    "rigid_world.phase7.ray.point.x",
    "rigid_world.phase7.ray.point.y",
    "rigid_world.phase7.origin_shift.x",
    "rigid_world.phase7.origin_shift.y",
];

const PHASE7_ULP_PATHS: &[&str] = &[
    "rigid_world.phase7.body.transform.angle",
    "rigid_world.phase7.body.angular_velocity",
    "rigid_world.phase7.body.linear_damping",
    "rigid_world.phase7.body.angular_damping",
    "rigid_world.phase7.body.gravity_scale",
    "rigid_world.phase7.contact.normal_impulse",
    "rigid_world.phase7.contact.tangent_impulse",
    "rigid_world.phase7.ray.fraction",
    "rigid_world.phase7.ray.normal.x",
    "rigid_world.phase7.ray.normal.y",
];

const PHASE8_STRUCTURAL_PATHS: &[&str] = &[
    "rigid_world.phase8.observations.order",
    "rigid_world.phase8.joint.id",
    "rigid_world.phase8.joint.kind",
    "rigid_world.phase8.joint.body_ids",
    "rigid_world.phase8.joint.collide_connected",
    "rigid_world.phase8.joint.dependencies.order",
    "rigid_world.phase8.joint.branch_state",
    "rigid_world.phase8.rope.id",
    "rigid_world.phase8.rope.vertex_count",
    "rigid_world.phase8.lifecycle.order",
    "rigid_world.phase8.lifecycle.kind",
    "rigid_world.phase8.lifecycle.identity",
    "rigid_world.phase8.lifecycle.multiplicity",
    "rigid_world.phase8.reconstruction.order",
    "rigid_world.phase8.reconstruction.kind",
    "rigid_world.phase8.reconstruction.support",
    "rigid_world.phase8.reconstruction.dependencies.order",
    "rigid_world.phase8.diagnostics.counts",
    "rigid_world.phase8.dump.order",
    "rigid_world.phase8.field.presence",
];

const PHASE8_EXACT_BITS_PATHS: &[&str] = &[
    "rigid_world.phase8.joint.configuration.bits",
    "rigid_world.phase8.rope.configuration.bits",
    "rigid_world.phase8.filter.directive.bits",
    "rigid_world.phase8.pre_solve.friction.bits",
    "rigid_world.phase8.pre_solve.restitution.bits",
    "rigid_world.phase8.pre_solve.tangent_speed.bits",
];

const PHASE8_ABSOLUTE_RELATIVE_PATHS: &[&str] = &[
    "rigid_world.phase8.joint.anchor.x",
    "rigid_world.phase8.joint.anchor.y",
    "rigid_world.phase8.joint.reaction_force.x",
    "rigid_world.phase8.joint.reaction_force.y",
    "rigid_world.phase8.rope.vertex.x",
    "rigid_world.phase8.rope.vertex.y",
    "rigid_world.phase8.joint.coordinate",
    "rigid_world.phase8.joint.speed",
    "rigid_world.phase8.joint.reaction_torque",
];

const PHASE8_ULP_PATHS: &[&str] = &["rigid_world.phase8.rope.angle"];

const PHASE8_ABSOLUTE_PATHS: &[&str] = &["rigid_world.phase8.diagnostics.tree_quality"];

/// Strict closed comparison profile for Phase 8 rigid evidence.
mod phase6;
mod phase7;
mod phase8;

#[cfg(test)]
use phase6::render_phase6_policy_presentation;
pub use phase6::{Phase6PolicyError, Phase6PolicyProfile};
pub use phase7::{Phase7PolicyError, Phase7PolicyProfile};
pub use phase8::{Phase8PolicyError, Phase8PolicyProfile};

#[cfg(test)]
mod tests;
