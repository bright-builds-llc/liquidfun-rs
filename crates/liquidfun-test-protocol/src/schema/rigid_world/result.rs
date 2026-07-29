use super::*;

pub(super) fn rigid_world_timeline_result_schema() -> Value {
    closed_record(
        &json!({
            "witness_family": { "enum": witness_families() },
            "checkpoints": {
                "items": checkpoint_result_schema(),
                "maxItems": 64,
                "minItems": 1,
                "type": "array"
            }
        }),
        &["witness_family", "checkpoints"],
    )
}

pub(super) fn checkpoint_result_schema() -> Value {
    closed_record(
        &json!({
            "checkpoint_id": semantic_id_schema(),
            "phase": bounded_string_schema(),
            "counts": expected_counts_schema(),
            "bodies": { "items": body_snapshot_schema(), "maxItems": 64, "type": "array" },
            "fixtures": { "items": fixture_snapshot_schema(), "maxItems": 128, "type": "array" },
            "contacts": { "items": contact_result_schema(), "maxItems": 128, "type": "array" },
            "events": { "items": event_schema(), "maxItems": 256, "type": "array" },
            "destructions": { "items": destruction_schema(), "maxItems": 256, "type": "array" },
            "observations": { "items": world_observation_schema(), "maxItems": 256, "type": "array" }
        }),
        &[
            "checkpoint_id",
            "phase",
            "counts",
            "bodies",
            "fixtures",
            "contacts",
            "events",
            "destructions",
        ],
    )
}

pub(super) fn wake_policy_schema() -> Value {
    json!({ "enum": enum_values(&[RigidWakePolicy::Wake, RigidWakePolicy::PreserveSleep]) })
}

pub(super) fn aabb_schema() -> Value {
    closed_record(
        &json!({
            "lower": schema_ref("vec2_bits"),
            "upper": schema_ref("vec2_bits")
        }),
        &["lower", "upper"],
    )
}

pub(super) fn fixture_child_selector_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "child_index": uint32_schema()
        }),
        &["fixture_id", "child_index"],
    )
}

pub(super) fn query_directive_rule_schema() -> Value {
    closed_record(
        &json!({
            "target": fixture_child_selector_schema(),
            "directive": { "enum": enum_values(&[RigidQueryDirective::Continue, RigidQueryDirective::Terminate]) }
        }),
        &["target", "directive"],
    )
}

pub(super) fn ray_directive_rule_schema() -> Value {
    closed_record(
        &json!({
            "target": fixture_child_selector_schema(),
            "directive": {
                "oneOf": [
                    tagged_probe_input("ignore", &json!({}), &[]),
                    tagged_probe_input("terminate", &json!({}), &[]),
                    tagged_probe_input("continue", &json!({}), &[]),
                    tagged_probe_input("clip", &json!({ "fraction_bits": float_bits_schema() }), &["fraction_bits"])
                ]
            }
        }),
        &["target", "directive"],
    )
}

pub(super) fn world_observation_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("body_state", &json!({ "state": body_control_snapshot_schema() }), &["state"]),
            tagged_probe_input("step", &json!({ "outcome": step_outcome_schema() }), &["outcome"]),
            tagged_probe_input("query", &json!({ "observation": query_observation_schema() }), &["observation"]),
            tagged_probe_input("ray_cast", &json!({ "observation": ray_observation_schema() }), &["observation"]),
            tagged_probe_input("origin_shift", &json!({ "shift": schema_ref("rigid_vec2_bits") }), &["shift"]),
            tagged_probe_input("joint", &json!({ "snapshot": joint_snapshot_schema() }), &["snapshot"]),
            tagged_probe_input("rope", &json!({ "snapshot": rope_snapshot_schema() }), &["snapshot"]),
            tagged_probe_input("lifecycle", &json!({ "event": lifecycle_observation_schema() }), &["event"]),
            tagged_probe_input("reconstruction", &json!({ "record": reconstruction_observation_schema() }), &["record"]),
            tagged_probe_input("diagnostics", &json!({ "snapshot": diagnostics_observation_schema() }), &["snapshot"])
            ,tagged_probe_input("particle", &json!({ "observation": particle_observation_schema() }), &["observation"]),
            tagged_probe_input("particle_group", &json!({ "observation": phase10_observation_schema() }), &["observation"])
        ]
    })
}

pub(super) fn body_control_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "body_id": semantic_id_schema(),
            "linear_velocity": schema_ref("rigid_vec2_bits"),
            "angular_velocity_bits": float_bits_schema(),
            "awake": { "type": "boolean" },
            "bullet": { "type": "boolean" },
            "sleeping_allowed": { "type": "boolean" },
            "fixed_rotation": { "type": "boolean" },
            "linear_damping_bits": float_bits_schema(),
            "angular_damping_bits": float_bits_schema(),
            "gravity_scale_bits": float_bits_schema()
        }),
        &[
            "body_id",
            "linear_velocity",
            "angular_velocity_bits",
            "awake",
            "bullet",
            "sleeping_allowed",
            "fixed_rotation",
            "linear_damping_bits",
            "angular_damping_bits",
            "gravity_scale_bits",
        ],
    )
}

pub(super) fn step_outcome_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("completed", &json!({
                "completion": { "enum": enum_values(&[RigidStepCompletion::Complete, RigidStepCompletion::ContinuousPending]) }
            }), &["completion"]),
            tagged_probe_input("partial", &json!({
                "classification": { "enum": enum_values(&[RigidPartialProgressClassification::ContinuousWorkBudgetExhausted]) }
            }), &["classification"])
        ]
    })
}

pub(super) fn query_observation_schema() -> Value {
    closed_record(
        &json!({
            "completion": { "enum": enum_values(&[RigidQueryCompletion::Exhausted, RigidQueryCompletion::Terminated]) },
            "occurrences": { "items": fixture_child_selector_schema(), "maxItems": 256, "type": "array" }
        }),
        &["completion", "occurrences"],
    )
}

pub(super) fn ray_observation_schema() -> Value {
    closed_record(
        &json!({
            "completion": { "enum": enum_values(&[RigidRayCompletion::Exhausted, RigidRayCompletion::Terminated]) },
            "final_max_fraction_bits": float_bits_schema(),
            "hits": { "items": ray_hit_schema(), "maxItems": 256, "type": "array" }
        }),
        &["completion", "final_max_fraction_bits", "hits"],
    )
}

pub(super) fn ray_hit_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "child_index": uint32_schema(),
            "point": schema_ref("rigid_vec2_bits"),
            "normal": schema_ref("rigid_vec2_bits"),
            "fraction_bits": float_bits_schema()
        }),
        &[
            "fixture_id",
            "child_index",
            "point",
            "normal",
            "fraction_bits",
        ],
    )
}

pub(super) fn joint_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "joint_id": semantic_id_schema(),
            "joint_kind": { "enum": enum_values(&RigidJointKind::ALL) },
            "body_a_id": semantic_id_schema(),
            "body_b_id": semantic_id_schema(),
            "collide_connected": { "type": "boolean" },
            "dependencies": { "items": semantic_id_schema(), "maxItems": 2, "type": "array" },
            "branch_state": { "enum": enum_values(&[
                RigidJointBranchState::Inactive,
                RigidJointBranchState::AtLower,
                RigidJointBranchState::AtUpper,
                RigidJointBranchState::Equal,
                RigidJointBranchState::Active,
            ]) },
            "coordinate_bits": float_bits_schema(),
            "speed_bits": float_bits_schema(),
            "reaction_force": schema_ref("rigid_vec2_bits"),
            "reaction_torque_bits": float_bits_schema()
        }),
        &[
            "joint_id",
            "joint_kind",
            "body_a_id",
            "body_b_id",
            "collide_connected",
            "dependencies",
            "branch_state",
            "coordinate_bits",
            "speed_bits",
            "reaction_force",
            "reaction_torque_bits",
        ],
    )
}

pub(super) fn rope_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "rope_id": semantic_id_schema(),
            "vertices": { "items": schema_ref("rigid_vec2_bits"), "maxItems": RIGID_WORLD_MAXIMUM_ROPE_VERTICES, "minItems": 3, "type": "array" }
        }),
        &["rope_id", "vertices"],
    )
}

pub(super) fn lifecycle_observation_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "kind": { "enum": enum_values(&[
                RigidLifecycleObservationKind::FilterDecision,
                RigidLifecycleObservationKind::ContactCreated,
                RigidLifecycleObservationKind::BeginContact,
                RigidLifecycleObservationKind::PreSolve,
                RigidLifecycleObservationKind::PostSolve,
                RigidLifecycleObservationKind::EndContact,
                RigidLifecycleObservationKind::ContactDestroyed,
                RigidLifecycleObservationKind::JointGoodbye,
                RigidLifecycleObservationKind::FixtureGoodbye,
                RigidLifecycleObservationKind::BodyDestroyed,
            ]) },
            "maybe_contact": { "oneOf": [contact_identity_schema(), { "type": "null" }] },
            "maybe_entity_id": { "oneOf": [semantic_id_schema(), { "type": "null" }] }
        }),
        &["ordinal", "kind", "maybe_contact", "maybe_entity_id"],
    )
}

pub(super) fn reconstruction_observation_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "kind": { "enum": enum_values(&[RigidReconstructionKind::Body, RigidReconstructionKind::Fixture, RigidReconstructionKind::Joint]) },
            "entity_id": semantic_id_schema(),
            "support": { "enum": enum_values(&[RigidReconstructionSupport::Supported, RigidReconstructionSupport::UnsupportedMouseJoint]) },
            "dependency_ids": { "items": semantic_id_schema(), "maxItems": 2, "type": "array" }
        }),
        &["ordinal", "kind", "entity_id", "support", "dependency_ids"],
    )
}

pub(super) fn diagnostics_observation_schema() -> Value {
    closed_record(
        &json!({
            "body_count": uint32_schema(),
            "fixture_count": uint32_schema(),
            "joint_count": uint32_schema(),
            "contact_count": uint32_schema(),
            "tree_height": uint32_schema(),
            "tree_max_balance": uint32_schema(),
            "tree_quality_bits": float_bits_schema()
        }),
        &[
            "body_count",
            "fixture_count",
            "joint_count",
            "contact_count",
            "tree_height",
            "tree_max_balance",
            "tree_quality_bits",
        ],
    )
}

pub(super) fn body_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "body_id": semantic_id_schema(),
            "body_kind": { "enum": body_kinds() },
            "transform": schema_ref("rigid_transform_bits"),
            "active": { "type": "boolean" },
            "linear_velocity": schema_ref("rigid_vec2_bits"),
            "angular_velocity_bits": float_bits_schema(),
            "mass_bits": float_bits_schema(),
            "local_center": schema_ref("rigid_vec2_bits"),
            "inertia_bits": float_bits_schema()
        }),
        &[
            "body_id",
            "body_kind",
            "transform",
            "active",
            "linear_velocity",
            "angular_velocity_bits",
            "mass_bits",
            "local_center",
            "inertia_bits",
        ],
    )
}

pub(super) fn fixture_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "owner_body_id": semantic_id_schema(),
            "sensor": { "type": "boolean" },
            "density_bits": float_bits_schema(),
            "friction_bits": float_bits_schema(),
            "restitution_bits": float_bits_schema(),
            "filter": filter_schema()
        }),
        &[
            "fixture_id",
            "owner_body_id",
            "sensor",
            "density_bits",
            "friction_bits",
            "restitution_bits",
            "filter",
        ],
    )
}

pub(super) fn contact_result_schema() -> Value {
    closed_record(
        &json!({
            "identity": contact_identity_schema(),
            "touching": { "type": "boolean" },
            "enabled": { "type": "boolean" },
            "sensor": { "type": "boolean" },
            "mixed_friction_bits": float_bits_schema(),
            "mixed_restitution_bits": float_bits_schema(),
            "maybe_manifold": { "oneOf": [manifold_schema(), { "type": "null" }] }
        }),
        &[
            "identity",
            "touching",
            "enabled",
            "sensor",
            "mixed_friction_bits",
            "mixed_restitution_bits",
            "maybe_manifold",
        ],
    )
}

pub(super) fn manifold_schema() -> Value {
    closed_record(
        &json!({
            "manifold_kind": { "enum": enum_values(&[RigidManifoldKind::Circles, RigidManifoldKind::FaceA, RigidManifoldKind::FaceB]) },
            "local_normal": schema_ref("rigid_vec2_bits"),
            "local_point": schema_ref("rigid_vec2_bits"),
            "points": { "items": manifold_point_schema(), "maxItems": 2, "type": "array" }
        }),
        &["manifold_kind", "local_normal", "local_point", "points"],
    )
}

pub(super) fn manifold_point_schema() -> Value {
    closed_record(
        &json!({
            "point": schema_ref("rigid_vec2_bits"),
            "feature": closed_record(
                &json!({
                    "index_a": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
                    "index_b": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
                    "kind_a": { "enum": enum_values(&[RigidFeatureKind::Vertex, RigidFeatureKind::Face]) },
                    "kind_b": { "enum": enum_values(&[RigidFeatureKind::Vertex, RigidFeatureKind::Face]) }
                }),
                &["index_a", "index_b", "kind_a", "kind_b"],
            ),
            "normal_impulse_bits": float_bits_schema(),
            "tangent_impulse_bits": float_bits_schema()
        }),
        &[
            "point",
            "feature",
            "normal_impulse_bits",
            "tangent_impulse_bits",
        ],
    )
}

pub(super) fn event_schema() -> Value {
    closed_record(
        &json!({
            "kind": { "enum": enum_values(&[
                RigidContactEventKind::Created,
                RigidContactEventKind::Begin,
                RigidContactEventKind::Persist,
                RigidContactEventKind::End,
                RigidContactEventKind::PreSolve,
                RigidContactEventKind::PostSolve,
                RigidContactEventKind::Destroyed,
            ]) },
            "contact": contact_identity_schema()
        }),
        &["kind", "contact"],
    )
}

pub(super) fn destruction_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("contact", &json!({ "contact": contact_identity_schema() }), &["contact"]),
            tagged_probe_input("fixture", &json!({ "fixture_id": semantic_id_schema() }), &["fixture_id"]),
            tagged_probe_input("body", &json!({ "body_id": semantic_id_schema() }), &["body_id"]),
            tagged_probe_input("joint", &json!({ "joint_id": semantic_id_schema() }), &["joint_id"])
        ]
    })
}

pub(super) fn witness_families() -> Value {
    enum_values(&RigidWorldWitnessFamily::ALL)
}

pub(super) fn witnesses() -> Value {
    let witnesses = RigidWorldWitnessFamily::ALL
        .into_iter()
        .flat_map(RigidWorldWitnessFamily::required_witnesses)
        .copied()
        .collect::<Vec<RigidWorldWitness>>();
    enum_values(&witnesses)
}

pub(super) fn body_kinds() -> Value {
    enum_values(&[
        RigidBodyKind::Static,
        RigidBodyKind::Kinematic,
        RigidBodyKind::Dynamic,
    ])
}

pub(super) fn enum_values<T: Serialize>(values: &[T]) -> Value {
    serde_json::to_value(values).expect("closed protocol enum serialization cannot fail")
}
