use std::collections::{HashMap, HashSet};

use super::super::PassId;
use super::{PASS_COUNT, PASS_GRAPH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WitnessPassId {
    Known(PassId),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PassWitness {
    id: WitnessPassId,
    control: &'static str,
    activation: &'static str,
    interaction: &'static str,
}

const fn witness(
    id: PassId,
    control: &'static str,
    activation: &'static str,
    interaction: &'static str,
) -> PassWitness {
    PassWitness {
        id: WitnessPassId::Known(id),
        control,
        activation,
        interaction,
    }
}

const PASS_WITNESSES: [PassWitness; PASS_COUNT] = [
    witness(
        PassId::Lifetime,
        "world_step_compacts_expired_particles_even_when_the_system_is_paused",
        "requested_particle_destructions_follow_newest_system_first_order",
        "particle_event_limit_failure_restores_lifetime_and_storage_for_retry",
    ),
    witness(
        PassId::ZombieCompaction,
        "unrequested_particle_destruction_compacts_without_fabricating_a_callback",
        "paused_step_compacts_an_explicit_zombie_and_journals_only_requested_occurrences",
        "maximum_count_creation_compacts_immediately_and_preserves_the_replacement",
    ),
    witness(
        PassId::RefreshParticleFlags,
        "private_pass_trace_gate_families_omit_only_inactive_passes",
        "private_pass_trace_refresh_reactive_and_zombie_admission_is_explicit",
        "reactive_topology_activates_once_and_clears_only_marked_flags",
    ),
    witness(
        PassId::RefreshGroupFlags,
        "private_pass_trace_gate_families_omit_only_inactive_passes",
        "private_pass_trace_refresh_reactive_and_zombie_admission_is_explicit",
        "solid_depth_mixed_groups_update_only_scheduled_members",
    ),
    witness(
        PassId::PauseGate,
        "empty_particle_world_has_no_solver_effect",
        "paused_nonempty_system_preserves_particle_state",
        "world_step_compacts_expired_particles_even_when_the_system_is_paused",
    ),
    witness(
        PassId::ParticleContacts,
        "particle_contacts_replace_in_candidate_order_and_empty_is_a_control",
        "contact_generation_computes_source_fields_without_calling_an_unflagged_filter",
        "particle_contact_failure_is_an_exact_no_diff_with_relative_slots",
    ),
    witness(
        PassId::BodyContacts,
        "stationary_collision_control_with_no_filtered_hits_is_exact",
        "body_contacts_update_stuck_state_without_reordering_contacts",
        "missing_body_fails_before_particle_velocity_mutation",
    ),
    witness(
        PassId::Weight,
        "empty_and_no_contact_pressure_family_controls_are_byte_identical",
        "weight_accumulates_body_rows_before_particle_rows",
        "pressure_traverses_body_contacts_before_particle_contacts",
    ),
    witness(
        PassId::SolidDepth,
        "retained_empty_group_does_not_activate_depth",
        "solid_depth_mixed_groups_update_only_scheduled_members",
        "solid_has_control_activation_zero_mixed_and_deterministic_witnesses",
    ),
    witness(
        PassId::ReactiveTopology,
        "private_pass_trace_gate_families_omit_only_inactive_passes",
        "reactive_topology_activates_once_and_clears_only_marked_flags",
        "split_preserves_original_first_and_allocates_later_components_in_source_order",
    ),
    witness(
        PassId::Force,
        "force_applies_once_and_inactive_second_call_is_an_exact_no_diff",
        "force_applies_once_and_inactive_second_call_is_an_exact_no_diff",
        "invalid_force_candidate_preserves_pending_state_and_velocities",
    ),
    witness(
        PassId::Viscous,
        "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
    ),
    witness(
        PassId::Repulsive,
        "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
    ),
    witness(
        PassId::Powder,
        "powder_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "powder_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile",
    ),
    witness(
        PassId::Tensile,
        "tensile_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "tensile_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile",
    ),
    witness(
        PassId::Solid,
        "solid_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "solid_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "solid_depth_mixed_groups_update_only_scheduled_members",
    ),
    witness(
        PassId::ColorMixing,
        "color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses",
        "color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses",
    ),
    witness(
        PassId::Gravity,
        "zero_timestep_and_empty_gravity_are_exact_controls",
        "gravity_uses_substep_scale_for_all_mixed_flags",
        "valid_one_and_two_iteration_steps_integrate_the_same_horizon",
    ),
    witness(
        PassId::StaticPressure,
        "static_pressure_zero_strength_and_no_contacts_are_exact_controls",
        "static_pressure_uses_configured_strength_relaxation_and_iterations",
        "extra_damping_checks_each_particle_gate_independently",
    ),
    witness(
        PassId::Pressure,
        "pressure_zero_coefficient_is_byte_identical_without_static_pressure",
        "pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile",
        "pressure_traverses_body_contacts_before_particle_contacts",
    ),
    witness(
        PassId::Damping,
        "empty_and_no_contact_pressure_family_controls_are_byte_identical",
        "damping_preserves_particle_contact_traversal_order",
        "damping_preserves_sequential_body_contact_updates",
    ),
    witness(
        PassId::ExtraDamping,
        "extra_damping_checks_each_particle_gate_independently",
        "extra_damping_checks_each_particle_gate_independently",
        "static_pressure_uses_configured_strength_relaxation_and_iterations",
    ),
    witness(
        PassId::Elastic,
        "empty_and_inactive_topology_are_exact_controls",
        "elastic_uses_stored_offsets_and_default_quarter_strength",
        "retargeted_topology_preserves_join_split_results_without_rebuilding_rest",
    ),
    witness(
        PassId::Spring,
        "empty_and_inactive_topology_are_exact_controls",
        "spring_uses_stored_rest_distance_and_default_quarter_strength",
        "spring_interaction_observes_source_record_order",
    ),
    witness(
        PassId::LimitVelocity,
        "limit_velocity_preserves_exact_threshold_and_clamps_only_above_it",
        "limit_velocity_preserves_exact_threshold_and_clamps_only_above_it",
        "storage_shell_commits_one_finite_velocity_candidate",
    ),
    witness(
        PassId::RigidDamping,
        "non_rigid_group_is_an_exact_control",
        "two_rigid_groups_exchange_damping_in_source_contact_order",
        "rigid_body_contact_emits_candidate_without_mutating_identity",
    ),
    witness(
        PassId::Barrier,
        "collapsed_barrier_pair_preserves_probe_backed_finite_noop",
        "barrier_activation_stops_crossing_particle_and_preserves_follow_up_force",
        "mixed_rigid_barrier_interaction_preserves_ids_memberships_and_order",
    ),
    witness(
        PassId::Collision,
        "stationary_collision_control_with_no_filtered_hits_is_exact",
        "collision_applies_filtered_hit_in_stable_order_and_records_force",
        "collision_start_uses_previous_transform_only_on_first_iteration",
    ),
    witness(
        PassId::Rigid,
        "non_rigid_group_is_an_exact_control",
        "rigid_statistics_and_projection_preserve_identity_order_and_association",
        "translated_and_rotated_projection_uses_source_transform_order",
    ),
    witness(
        PassId::Wall,
        "barrier_wall_endpoints_are_zeroed_before_crossing_scan",
        "wall_targets_only_wall_particles_after_rigid_projection",
        "mixed_rigid_barrier_interaction_preserves_ids_memberships_and_order",
    ),
    witness(
        PassId::Integrate,
        "empty_particle_world_has_no_solver_effect",
        "integration_occurs_exactly_once_and_only_after_wall",
        "valid_one_and_two_iteration_steps_integrate_the_same_horizon",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FlagKey {
    Water,
    Zombie,
    Wall,
    Spring,
    Elastic,
    Viscous,
    Powder,
    Tensile,
    ColorMixing,
    DestructionListener,
    Barrier,
    StaticPressure,
    Reactive,
    Repulsive,
    FixtureContactListener,
    ParticleContactListener,
    FixtureContactFilter,
    ParticleContactFilter,
    SolidGroup,
    RigidGroup,
    CanBeEmptyGroup,
    WillBeDestroyedInternal,
    NeedsUpdateDepthInternal,
}

const FLAG_KEYS: [FlagKey; 23] = [
    FlagKey::Water,
    FlagKey::Zombie,
    FlagKey::Wall,
    FlagKey::Spring,
    FlagKey::Elastic,
    FlagKey::Viscous,
    FlagKey::Powder,
    FlagKey::Tensile,
    FlagKey::ColorMixing,
    FlagKey::DestructionListener,
    FlagKey::Barrier,
    FlagKey::StaticPressure,
    FlagKey::Reactive,
    FlagKey::Repulsive,
    FlagKey::FixtureContactListener,
    FlagKey::ParticleContactListener,
    FlagKey::FixtureContactFilter,
    FlagKey::ParticleContactFilter,
    FlagKey::SolidGroup,
    FlagKey::RigidGroup,
    FlagKey::CanBeEmptyGroup,
    FlagKey::WillBeDestroyedInternal,
    FlagKey::NeedsUpdateDepthInternal,
];

#[derive(Debug, Clone, Copy)]
struct FlagWitness {
    key: FlagKey,
    control: &'static str,
    activation: &'static str,
    interaction: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclaredFlagKey {
    Known(FlagKey),
    Unknown,
}

#[derive(Debug, Clone, Copy)]
struct FlagDeclaration {
    key: DeclaredFlagKey,
    control: &'static str,
    activation: &'static str,
    interaction: &'static str,
}

impl From<FlagWitness> for FlagDeclaration {
    fn from(witness: FlagWitness) -> Self {
        Self {
            key: DeclaredFlagKey::Known(witness.key),
            control: witness.control,
            activation: witness.activation,
            interaction: witness.interaction,
        }
    }
}

const FLAG_WITNESSES: [FlagWitness; 23] = [
    FlagWitness {
        key: FlagKey::Water,
        control: "public_water_control_remains_zero_valued",
        activation: "valid_one_and_two_iteration_steps_integrate_the_same_horizon",
        interaction: "gravity_uses_substep_scale_for_all_mixed_flags",
    },
    FlagWitness {
        key: FlagKey::Zombie,
        control: "unrequested_particle_destruction_compacts_without_fabricating_a_callback",
        activation: "particle_created_with_zombie_compacts_on_next_fresh_positive_step",
        interaction: "paused_step_compacts_an_explicit_zombie_and_journals_only_requested_occurrences",
    },
    FlagWitness {
        key: FlagKey::Wall,
        control: "public_particle_flags_round_trip_through_particle_creation",
        activation: "wall_targets_only_wall_particles_after_rigid_projection",
        interaction: "barrier_wall_endpoints_are_zeroed_before_crossing_scan",
    },
    FlagWitness {
        key: FlagKey::Spring,
        control: "empty_and_inactive_topology_are_exact_controls",
        activation: "spring_uses_stored_rest_distance_and_default_quarter_strength",
        interaction: "spring_interaction_observes_source_record_order",
    },
    FlagWitness {
        key: FlagKey::Elastic,
        control: "empty_and_inactive_topology_are_exact_controls",
        activation: "elastic_uses_stored_offsets_and_default_quarter_strength",
        interaction: "retargeted_topology_preserves_join_split_results_without_rebuilding_rest",
    },
    FlagWitness {
        key: FlagKey::Viscous,
        control: "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
        activation: "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "viscous_has_control_activation_zero_mixed_and_deterministic_witnesses",
    },
    FlagWitness {
        key: FlagKey::Powder,
        control: "powder_has_control_activation_zero_mixed_and_deterministic_witnesses",
        activation: "powder_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile",
    },
    FlagWitness {
        key: FlagKey::Tensile,
        control: "tensile_has_control_activation_zero_mixed_and_deterministic_witnesses",
        activation: "tensile_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "pressure_uses_configured_coefficient_and_suppresses_powder_and_tensile",
    },
    FlagWitness {
        key: FlagKey::ColorMixing,
        control: "color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses",
        activation: "color_mixing_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::DestructionListener,
        control: "unrequested_particle_destruction_compacts_without_fabricating_a_callback",
        activation: "requested_particle_destructions_follow_newest_system_first_order",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::Barrier,
        control: "collapsed_barrier_pair_preserves_probe_backed_finite_noop",
        activation: "barrier_activation_stops_crossing_particle_and_preserves_follow_up_force",
        interaction: "mixed_rigid_barrier_interaction_preserves_ids_memberships_and_order",
    },
    FlagWitness {
        key: FlagKey::StaticPressure,
        control: "static_pressure_zero_strength_and_no_contacts_are_exact_controls",
        activation: "static_pressure_uses_configured_strength_relaxation_and_iterations",
        interaction: "extra_damping_checks_each_particle_gate_independently",
    },
    FlagWitness {
        key: FlagKey::Reactive,
        control: "private_pass_trace_gate_families_omit_only_inactive_passes",
        activation: "reactive_topology_activates_once_and_clears_only_marked_flags",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::Repulsive,
        control: "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
        activation: "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "repulsive_has_control_activation_zero_mixed_and_deterministic_witnesses",
    },
    FlagWitness {
        key: FlagKey::FixtureContactListener,
        control: "contact_generation_computes_source_fields_without_calling_an_unflagged_filter",
        activation: "rigid_contact_effects_precede_particle_destruction_in_the_shared_journal",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::ParticleContactListener,
        control: "contact_generation_computes_source_fields_without_calling_an_unflagged_filter",
        activation: "late_contact_journal_failure_rolls_back_particle_state",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::FixtureContactFilter,
        control: "contact_generation_computes_source_fields_without_calling_an_unflagged_filter",
        activation: "contact_filter_is_invoked_only_for_flagged_contacts_and_can_reject_them",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::ParticleContactFilter,
        control: "contact_generation_computes_source_fields_without_calling_an_unflagged_filter",
        activation: "contact_filter_is_invoked_only_for_flagged_contacts_and_can_reject_them",
        interaction: "public_particle_flags_round_trip_through_particle_creation",
    },
    FlagWitness {
        key: FlagKey::SolidGroup,
        control: "public_group_flags_hide_private_bits_and_preserve_unknown_public_bits",
        activation: "solid_has_control_activation_zero_mixed_and_deterministic_witnesses",
        interaction: "solid_depth_mixed_groups_update_only_scheduled_members",
    },
    FlagWitness {
        key: FlagKey::RigidGroup,
        control: "public_group_flags_hide_private_bits_and_preserve_unknown_public_bits",
        activation: "rigid_statistics_and_projection_preserve_identity_order_and_association",
        interaction: "mixed_rigid_barrier_interaction_preserves_ids_memberships_and_order",
    },
    FlagWitness {
        key: FlagKey::CanBeEmptyGroup,
        control: "flags_and_retained_empty_shell_follow_explicit_lifecycle_rules",
        activation: "can_be_empty_group_retains_exact_zero_state_and_accepts_a_later_append",
        interaction: "public_group_flags_hide_private_bits_and_preserve_unknown_public_bits",
    },
    FlagWitness {
        key: FlagKey::WillBeDestroyedInternal,
        control: "public_group_flags_hide_private_bits_and_preserve_unknown_public_bits",
        activation: "ordinary_group_destruction_is_deferred_and_callbacks_are_source_ordered_once",
        interaction: "retained_or_deferred_empty_records_require_exact_zero_statistics",
    },
    FlagWitness {
        key: FlagKey::NeedsUpdateDepthInternal,
        control: "retained_empty_group_does_not_activate_depth",
        activation: "solid_depth_mixed_groups_update_only_scheduled_members",
        interaction: "public_group_flags_hide_private_bits_and_preserve_unknown_public_bits",
    },
];

fn validate_pass_witnesses(entries: &[PassWitness]) -> Result<(), &'static str> {
    if entries.len() != PASS_COUNT {
        return Err("wrong pass witness cardinality");
    }
    let mut seen = HashMap::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let WitnessPassId::Known(id) = entry.id else {
            return Err("unknown pass witness");
        };
        if !PASS_GRAPH.iter().any(|descriptor| descriptor.id == id) {
            return Err("unknown pass witness");
        }
        if seen.insert(id, index).is_some() {
            return Err("duplicate pass witness");
        }
        if entry.control.is_empty() || entry.activation.is_empty() || entry.interaction.is_empty() {
            return Err("unnamed pass witness");
        }
    }
    if PASS_GRAPH
        .iter()
        .any(|descriptor| !seen.contains_key(&descriptor.id))
    {
        return Err("missing pass witness");
    }
    Ok(())
}

fn validate_flag_witnesses(entries: &[FlagDeclaration]) -> Result<(), &'static str> {
    if entries.len() != FLAG_KEYS.len() {
        return Err("wrong flag witness cardinality");
    }
    let mut seen = HashSet::with_capacity(entries.len());
    for entry in entries {
        let DeclaredFlagKey::Known(key) = entry.key else {
            return Err("unknown flag witness");
        };
        if !FLAG_KEYS.contains(&key) {
            return Err("unknown flag witness");
        }
        if !seen.insert(key) {
            return Err("duplicate flag witness");
        }
        if entry.control.is_empty() || entry.activation.is_empty() || entry.interaction.is_empty() {
            return Err("unnamed flag witness");
        }
    }
    if FLAG_KEYS.iter().any(|key| !seen.contains(key)) {
        return Err("missing flag witness");
    }
    Ok(())
}

#[test]
fn witness_registries_are_closed_complete_and_named() {
    // Arrange / Act
    let pass_result = validate_pass_witnesses(&PASS_WITNESSES);
    let flags = FLAG_WITNESSES.map(FlagDeclaration::from);
    let flag_result = validate_flag_witnesses(&flags);

    // Assert
    assert_eq!(PASS_WITNESSES.len(), 31);
    assert_eq!(FLAG_WITNESSES.len(), 23);
    assert_eq!(pass_result, Ok(()));
    assert_eq!(flag_result, Ok(()));
}

#[test]
fn flag_registry_rejects_missing_duplicate_and_unknown_entries() {
    // Arrange
    let declarations = FLAG_WITNESSES.map(FlagDeclaration::from);
    let mut missing = declarations.to_vec();
    missing.pop();
    let mut duplicate = declarations;
    duplicate[22] = duplicate[21];
    let mut unknown = declarations;
    unknown[22].key = DeclaredFlagKey::Unknown;

    // Act / Assert
    assert_eq!(
        validate_flag_witnesses(&missing),
        Err("wrong flag witness cardinality")
    );
    assert_eq!(
        validate_flag_witnesses(&duplicate),
        Err("duplicate flag witness")
    );
    assert_eq!(
        validate_flag_witnesses(&unknown),
        Err("unknown flag witness")
    );
}

#[test]
fn pass_registry_rejects_missing_duplicate_and_unknown_entries() {
    // Arrange
    let mut missing = PASS_WITNESSES.to_vec();
    missing.pop();
    let mut duplicate = PASS_WITNESSES;
    duplicate[30] = duplicate[29];
    let mut unknown = PASS_WITNESSES;
    unknown[30].id = WitnessPassId::Unknown;

    // Act / Assert
    assert_eq!(
        validate_pass_witnesses(&missing),
        Err("wrong pass witness cardinality")
    );
    assert_eq!(
        validate_pass_witnesses(&duplicate),
        Err("duplicate pass witness")
    );
    assert_eq!(
        validate_pass_witnesses(&unknown),
        Err("unknown pass witness")
    );
}
