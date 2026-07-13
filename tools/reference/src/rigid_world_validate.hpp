// Closed rigid-world witness and lifecycle validation.
inline const std::vector<std::string_view>& required_witnesses(
    RigidWitnessFamily value) {
  static const std::vector<std::string_view> non_colliding{
      "static_body_created", "kinematic_body_created", "dynamic_body_created",
      "fixtures_created", "body_inspected", "fixture_inspected",
      "body_transform_changed", "body_type_changed", "body_deactivated",
      "body_reactivated", "sensor_enabled", "sensor_disabled",
      "material_changed", "filter_changed", "density_changed_without_mass_reset",
      "mass_reset", "custom_mass_set", "static_kinematic_overlap_rejected",
      "kinematic_kinematic_overlap_rejected", "zero_contact_step",
      "fixture_destroyed", "body_destroyed"};
  static const std::vector<std::string_view> single_contact{
      "contact_created", "contact_begin", "contact_persisted", "manifold_active",
      "contact_solved", "warm_start_transferred", "sensor_touching",
      "sensor_without_manifold", "filter_removed_contact",
      "filter_recreated_contact", "deactivation_destroyed_contact",
      "reactivation_recreated_contact", "fixture_destroyed_contact",
      "body_cascade_end_ordered"};
  static const std::vector<std::string_view> body_control{
      "force_wake_policy", "force_preserve_sleep_policy", "impulse_wake_policy",
      "velocity_wake_policy", "damping_and_gravity_scale_applied",
      "fixed_rotation_applied", "automatic_force_clearing_applied",
      "manual_force_clearing_applied"};
  static const std::vector<std::string_view> island{
      "multi_contact_island_solved", "island_traversal_ordered",
      "warm_start_applied", "warm_start_disabled_then_stored"};
  static const std::vector<std::string_view> sleep{
      "sleeping_threshold_reached", "whole_island_slept", "mutation_woke_body",
      "contact_woke_island", "activation_preserved_sleep"};
  static const std::vector<std::string_view> ccd{
      "continuous_physics_prevented_tunneling", "disabled_continuous_physics_tunneled",
      "bullet_state_selected_continuous_contact", "continuous_step_completed",
      "sub_step_reported_pending", "sub_step_resume_completed",
      "continuous_transitions_ordered"};
  static const std::vector<std::string_view> budget{
      "continuous_budget_exhausted", "continuous_budget_state_coherent",
      "continuous_budget_resume_completed"};
  static const std::vector<std::string_view> query_ray{
      "query_preserved_duplicate_occurrences", "query_exhausted", "query_terminated",
      "query_explicit_filter_applied", "ray_missed", "ray_rejected_invalid_directive",
      "ray_ignored_hit", "ray_continued_without_clipping", "ray_clipped",
      "ray_terminated", "ray_nearest_hit_selected", "ray_equal_fraction_tie_set"};
  static const std::vector<std::string_view> origin{
      "origin_shift_rejected_while_locked", "origin_shift_rejected_non_finite",
      "origin_shift_rejected_overflow", "origin_shift_translated_bodies",
      "origin_shift_preserved_query_hits",
      "origin_shift_preserved_ray_fractions_and_normals",
      "origin_shift_preserved_topology"};
  if (value == RigidWitnessFamily::non_colliding) return non_colliding;
  if (value == RigidWitnessFamily::single_contact) return single_contact;
  if (value == RigidWitnessFamily::body_control) return body_control;
  if (value == RigidWitnessFamily::island_warm_start) return island;
  if (value == RigidWitnessFamily::sleeping_waking) return sleep;
  if (value == RigidWitnessFamily::continuous_collision) return ccd;
  if (value == RigidWitnessFamily::continuous_budget) return budget;
  if (value == RigidWitnessFamily::query_ray) return query_ray;
  return origin;
}

inline std::string action_name(const RigidAction& value) {
  static constexpr std::array<std::string_view, 38> names{
      "create_body", "create_fixture", "inspect_body", "inspect_fixture",
      "set_body_transform", "set_body_type", "set_body_active",
      "set_linear_velocity", "set_angular_velocity", "apply_force",
      "apply_torque", "apply_linear_impulse", "apply_angular_impulse",
      "set_body_damping", "set_gravity_scale", "set_fixed_rotation",
      "set_sleeping_allowed", "set_awake", "set_bullet",
      "set_fixture_sensor", "set_fixture_material", "set_fixture_filter",
      "set_fixture_density", "reset_mass_data", "set_custom_mass_data", "step",
      "set_world_gravity", "set_automatic_force_clearing", "set_warm_starting",
      "set_continuous_physics", "set_sub_stepping", "clear_forces",
      "configured_step", "query_aabb", "ray_cast", "shift_origin",
      "destroy_fixture", "destroy_body"};
  return std::string(names[value.index()]);
}

inline const std::vector<std::string_view>& required_actions(RigidWitnessFamily family) {
  static const std::vector<std::string_view> non_colliding{
      "create_body", "create_fixture", "inspect_body", "inspect_fixture",
      "set_body_transform", "set_body_type", "set_body_active",
      "set_fixture_sensor", "set_fixture_material", "set_fixture_filter",
      "set_fixture_density", "reset_mass_data", "set_custom_mass_data", "step",
      "destroy_fixture", "destroy_body"};
  static const std::vector<std::string_view> single_contact{
      "create_body", "create_fixture", "set_body_active", "set_fixture_sensor",
      "set_fixture_filter", "step", "destroy_fixture", "destroy_body"};
  static const std::vector<std::string_view> body_control{
      "create_body", "create_fixture", "set_linear_velocity", "set_angular_velocity",
      "apply_force", "apply_torque", "apply_linear_impulse", "apply_angular_impulse",
      "set_body_damping", "set_gravity_scale", "set_fixed_rotation",
      "set_sleeping_allowed", "set_awake", "set_world_gravity",
      "set_automatic_force_clearing", "clear_forces", "configured_step", "destroy_body"};
  static const std::vector<std::string_view> island{
      "create_body", "create_fixture", "set_warm_starting", "configured_step", "destroy_body"};
  static const std::vector<std::string_view> sleep{
      "create_body", "create_fixture", "set_body_active", "set_linear_velocity",
      "apply_force", "set_sleeping_allowed", "configured_step"};
  static const std::vector<std::string_view> ccd{
      "create_body", "create_fixture", "set_bullet", "set_continuous_physics",
      "set_sub_stepping", "set_linear_velocity", "configured_step"};
  static const std::vector<std::string_view> budget{
      "create_body", "create_fixture", "set_bullet", "configured_step"};
  static const std::vector<std::string_view> query_ray{
      "create_body", "create_fixture", "query_aabb", "ray_cast"};
  static const std::vector<std::string_view> origin{
      "create_body", "create_fixture", "query_aabb", "ray_cast", "shift_origin"};
  if (family == RigidWitnessFamily::non_colliding) return non_colliding;
  if (family == RigidWitnessFamily::single_contact) return single_contact;
  if (family == RigidWitnessFamily::body_control) return body_control;
  if (family == RigidWitnessFamily::island_warm_start) return island;
  if (family == RigidWitnessFamily::sleeping_waking) return sleep;
  if (family == RigidWitnessFamily::continuous_collision) return ccd;
  if (family == RigidWitnessFamily::continuous_budget) return budget;
  if (family == RigidWitnessFamily::query_ray) return query_ray;
  return origin;
}

inline void validate_timeline(RigidTimeline& timeline) {
  if (timeline.bodies.empty() || timeline.bodies.size() > 64 ||
      timeline.fixtures.empty() || timeline.fixtures.size() > 128 ||
      timeline.actions.empty() ||
      timeline.actions.size() > kRigidWorldMaximumActions ||
      timeline.checkpoints.empty() || timeline.checkpoints.size() > 64) {
    throw std::runtime_error("rigid timeline collection count outside reviewed bounds");
  }
  std::unordered_set<std::string> body_ids;
  for (const auto& body : timeline.bodies) {
    if (!body_ids.insert(body.id).second) throw std::runtime_error("duplicate body ID");
  }
  std::unordered_map<std::string, std::string> fixture_owners;
  for (const auto& fixture : timeline.fixtures) {
    if (!body_ids.count(fixture.owner_body_id)) throw std::runtime_error("invalid fixture owner");
    if (!fixture_owners.emplace(fixture.id, fixture.owner_body_id).second) {
      throw std::runtime_error("duplicate fixture ID");
    }
  }
  std::unordered_set<std::string> live_bodies;
  std::unordered_set<std::string> live_fixtures;
  std::unordered_set<std::string> created_bodies;
  std::unordered_set<std::string> created_fixtures;
  std::unordered_set<std::string> action_ids;
  std::unordered_set<std::string> action_kinds;
  std::unordered_map<std::string, std::size_t> action_positions;
  std::vector<std::pair<std::size_t, std::size_t>> live_counts;
  for (std::size_t index = 0; index < timeline.actions.size(); ++index) {
    const auto& record = timeline.actions[index];
    if (!action_ids.insert(record.id).second) throw std::runtime_error("duplicate action ID");
    if (record.phase.empty()) throw std::runtime_error("action phase must not be empty");
    action_positions.emplace(record.id, index);
    action_kinds.insert(action_name(record.action));
    std::visit(
        [&](const auto& current) {
          using T = std::decay_t<decltype(current)>;
          if constexpr (std::is_same_v<T, CreateBody>) {
            if (!body_ids.count(current.body_id) ||
                !created_bodies.insert(current.body_id).second ||
                !live_bodies.insert(current.body_id).second) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (std::is_same_v<T, CreateFixture>) {
            const auto owner = fixture_owners.find(current.fixture_id);
            if (owner == fixture_owners.end() || !live_bodies.count(owner->second) ||
                !created_fixtures.insert(current.fixture_id).second ||
                !live_fixtures.insert(current.fixture_id).second) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (std::is_same_v<T, DestroyFixture>) {
            if (live_fixtures.erase(current.fixture_id) != 1) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (std::is_same_v<T, DestroyBody>) {
            if (live_bodies.erase(current.body_id) != 1) {
              throw std::runtime_error("invalid rigid action order");
            }
            for (auto fixture = live_fixtures.begin(); fixture != live_fixtures.end();) {
              fixture = fixture_owners.at(*fixture) == current.body_id
                            ? live_fixtures.erase(fixture)
                            : std::next(fixture);
            }
          } else if constexpr (
              std::is_same_v<T, InspectBody> ||
              std::is_same_v<T, SetBodyTransform> ||
              std::is_same_v<T, SetBodyType> ||
              std::is_same_v<T, SetBodyActive> ||
              std::is_same_v<T, SetLinearVelocity> ||
              std::is_same_v<T, SetAngularVelocity> ||
              std::is_same_v<T, ApplyForce> ||
              std::is_same_v<T, ApplyTorque> ||
              std::is_same_v<T, ApplyLinearImpulse> ||
              std::is_same_v<T, ApplyAngularImpulse> ||
              std::is_same_v<T, SetBodyDamping> ||
              std::is_same_v<T, SetGravityScale> ||
              std::is_same_v<T, SetFixedRotation> ||
              std::is_same_v<T, SetSleepingAllowed> ||
              std::is_same_v<T, SetAwake> ||
              std::is_same_v<T, SetBullet> ||
              std::is_same_v<T, ResetMassData> ||
              std::is_same_v<T, SetCustomMassData>) {
            if (!live_bodies.count(current.body_id)) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (
              std::is_same_v<T, InspectFixture> ||
              std::is_same_v<T, SetFixtureSensor> ||
              std::is_same_v<T, SetFixtureMaterial> ||
              std::is_same_v<T, SetFixtureFilter> ||
              std::is_same_v<T, SetFixtureDensity>) {
            if (!live_fixtures.count(current.fixture_id)) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (std::is_same_v<T, QueryAabb>) {
            for (const auto& rule : current.rules) {
              if (!live_fixtures.count(rule.target.fixture_id)) {
                throw std::runtime_error("query directive references non-live fixture");
              }
            }
          } else if constexpr (std::is_same_v<T, RayCast>) {
            for (const auto& rule : current.rules) {
              if (!live_fixtures.count(rule.target.fixture_id)) {
                throw std::runtime_error("ray directive references non-live fixture");
              }
            }
          }
        },
        record.action);
    live_counts.emplace_back(live_bodies.size(), live_fixtures.size());
  }
  if (!live_bodies.empty() || !live_fixtures.empty() ||
      created_bodies.size() != body_ids.size() ||
      created_fixtures.size() != fixture_owners.size()) {
    throw std::runtime_error("invalid rigid action order");
  }
  for (const auto required : required_actions(timeline.family)) {
    if (!action_kinds.count(std::string(required))) {
      throw std::runtime_error("missing rigid action kind");
    }
  }
  std::unordered_set<std::string> checkpoint_ids;
  std::unordered_set<std::string> witnesses;
  std::size_t previous_position = 0;
  bool first = true;
  for (const auto& checkpoint : timeline.checkpoints) {
    if (!checkpoint_ids.insert(checkpoint.id).second) {
      throw std::runtime_error("duplicate checkpoint ID");
    }
    const auto found = action_positions.find(checkpoint.after_action_id);
    if (found == action_positions.end() || (!first && found->second <= previous_position)) {
      throw std::runtime_error("invalid checkpoint order");
    }
    first = false;
    previous_position = found->second;
    if (checkpoint.phase != timeline.actions[found->second].phase) {
      throw std::runtime_error("checkpoint phase mismatch");
    }
    if (checkpoint.counts.bodies != live_counts[found->second].first ||
        checkpoint.counts.fixtures != live_counts[found->second].second ||
        checkpoint.counts.manifold_points > checkpoint.counts.contacts * 2 ||
        (timeline.family == RigidWitnessFamily::non_colliding &&
         (checkpoint.counts.contacts != 0 || checkpoint.counts.manifold_points != 0)) ||
        (timeline.family == RigidWitnessFamily::single_contact &&
         checkpoint.counts.contacts > 1)) {
      throw std::runtime_error("expected checkpoint count mismatch");
    }
    for (const auto& transition : checkpoint.transitions) {
      if (!witnesses.insert(transition.witness).second) {
        throw std::runtime_error("duplicate witness");
      }
      if (transition.maybe_contact.has_value() &&
          (!fixture_owners.count(transition.maybe_contact->fixture_a_id) ||
           !fixture_owners.count(transition.maybe_contact->fixture_b_id) ||
           transition.maybe_contact->child_a != 0 ||
           transition.maybe_contact->child_b != 0)) {
        throw std::runtime_error("invalid contact identity");
      }
    }
  }
  const auto& required = required_witnesses(timeline.family);
  if (witnesses.size() != required.size() ||
      std::any_of(required.begin(), required.end(), [&](std::string_view witness) {
        return !witnesses.count(std::string(witness));
      })) {
    throw std::runtime_error("rigid witness registry is incomplete");
  }
}
