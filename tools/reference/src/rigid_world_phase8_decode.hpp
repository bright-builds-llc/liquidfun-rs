// Strict bounded decoding for the Phase 8 extension. This file is included
// inside liquidfun::reference::rigid_world_decode after the shared helpers.

inline constexpr std::size_t kRigidWorldMaximumRopes = 8;
inline constexpr std::size_t kRigidWorldMaximumRopeVertices = 128;

inline const std::array<std::string_view, 10>& phase8_family_names() {
  static const std::array<std::string_view, 10> names{
      "joint_definitions_and_mutations",
      "revolute_prismatic_limits_and_motors",
      "distance_pulley_mouse_constraints",
      "wheel_weld_friction_rope_motor_constraints",
      "gear_dependencies_and_four_body_solver",
      "mixed_joint_island_order_and_collision_suppression",
      "standalone_rope_evolution",
      "contact_filter_listener_and_pre_solve_timing",
      "destruction_listener_and_dependency_cascades",
      "diagnostic_reconstruction_and_dump_order"};
  return names;
}

inline bool phase8_known_action(std::string_view kind) {
  static const std::array<std::string_view, 51> kinds{
      "create_body", "create_fixture", "inspect_body", "inspect_fixture",
      "set_body_transform", "set_body_type", "set_body_active",
      "set_linear_velocity", "set_angular_velocity", "apply_force",
      "apply_torque", "apply_linear_impulse", "apply_angular_impulse",
      "set_body_damping", "set_gravity_scale", "set_fixed_rotation",
      "set_sleeping_allowed", "set_awake", "set_bullet",
      "set_fixture_sensor", "set_fixture_material", "set_fixture_filter",
      "set_fixture_density", "reset_mass_data", "set_custom_mass_data",
      "step", "set_world_gravity", "set_automatic_force_clearing",
      "set_warm_starting", "set_continuous_physics", "set_sub_stepping",
      "clear_forces", "configured_step", "query_aabb", "ray_cast",
      "shift_origin", "create_joint", "inspect_joint", "mutate_joint",
      "destroy_joint", "create_rope", "set_rope_angle", "step_rope",
      "inspect_rope", "destroy_rope", "set_contact_filter_directive",
      "set_pre_solve_directive", "request_reconstruction",
      "request_diagnostics", "destroy_fixture", "destroy_body"};
  return std::find(kinds.begin(), kinds.end(), kind) != kinds.end();
}

inline void validate_phase8_bits(const Json& value, std::string_view context) {
  if (value.is_object()) {
    for (const auto& [name, child] : value.items()) {
      if (name.size() >= 5 && name.substr(name.size() - 5) == "_bits" &&
          !child.is_null() && child.is_number()) {
        const auto bits = u32(child, context);
        if (name != "category_bits" && name != "mask_bits") {
          require_finite(bits, context);
        }
      } else {
        validate_phase8_bits(child, context);
      }
    }
    return;
  }
  if (value.is_array()) {
    for (const auto& child : value) validate_phase8_bits(child, context);
  }
}

inline void validate_phase8_joint(const Json& joint) {
  require_members(
      joint,
      {"joint_id", "body_a_id", "body_b_id", "collide_connected", "definition"},
      "Phase 8 joint declaration");
  static_cast<void>(id(member(joint, "joint_id", "joint"), "joint ID"));
  static_cast<void>(id(member(joint, "body_a_id", "joint"), "joint body A ID"));
  static_cast<void>(id(member(joint, "body_b_id", "joint"), "joint body B ID"));
  static_cast<void>(boolean(
      member(joint, "collide_connected", "joint"), "joint collide-connected"));
  const auto& definition = member(joint, "definition", "joint");
  const auto kind = text(member(definition, "kind", "joint definition"), "joint kind");
  static const std::array<std::string_view, 11> kinds{
      "revolute", "prismatic", "distance", "pulley", "mouse", "gear",
      "wheel", "weld", "friction", "rope", "motor"};
  if (std::find(kinds.begin(), kinds.end(), kind) == kinds.end()) {
    throw std::runtime_error("unsupported Phase 8 joint kind");
  }
  validate_phase8_bits(definition, "Phase 8 joint definition");
}

inline void validate_phase8_rope(const Json& rope) {
  require_members(
      rope,
      {"rope_id", "vertices", "masses_bits", "gravity", "damping_bits",
       "stretch_stiffness_bits", "bend_stiffness_bits"},
      "Phase 8 rope declaration");
  static_cast<void>(id(member(rope, "rope_id", "rope"), "rope ID"));
  const auto& vertices = member(rope, "vertices", "rope");
  const auto& masses = member(rope, "masses_bits", "rope");
  if (!vertices.is_array() || !masses.is_array() || vertices.size() < 3 ||
      vertices.size() > kRigidWorldMaximumRopeVertices ||
      vertices.size() != masses.size()) {
    throw std::runtime_error("Phase 8 rope vertex count outside reviewed bounds");
  }
  for (const auto& vertex : vertices) static_cast<void>(vec2(vertex, "rope vertex"));
  for (const auto& mass : masses) require_nonnegative(u32(mass, "rope mass"), "rope mass");
  static_cast<void>(vec2(member(rope, "gravity", "rope"), "rope gravity"));
  validate_phase8_bits(rope, "Phase 8 rope declaration");
}

inline void validate_phase8_timeline(
    const Json& timeline,
    std::string_view expected_family) {
  require_members(
      timeline,
      {"witness_family", "bodies", "fixtures", "joints", "ropes", "actions",
       "checkpoints"},
      "Phase 8 timeline");
  if (text(member(timeline, "witness_family", "timeline"), "witness family") !=
      expected_family) {
    throw std::runtime_error("Phase 8 witness registry is incomplete or out of order");
  }
  const auto& bodies = member(timeline, "bodies", "timeline");
  const auto& fixtures = member(timeline, "fixtures", "timeline");
  const auto joints = timeline.value("joints", Json::array());
  const auto ropes = timeline.value("ropes", Json::array());
  const auto& actions = member(timeline, "actions", "timeline");
  const auto& checkpoints = member(timeline, "checkpoints", "timeline");
  if (!bodies.is_array() || bodies.empty() || bodies.size() > 64 ||
      !fixtures.is_array() || fixtures.empty() || fixtures.size() > 128 ||
      !joints.is_array() || joints.size() > kRigidWorldMaximumJoints ||
      !ropes.is_array() || ropes.size() > kRigidWorldMaximumRopes ||
      !actions.is_array() || actions.empty() ||
      actions.size() > kRigidWorldMaximumActions || !checkpoints.is_array() ||
      checkpoints.empty() || checkpoints.size() > 64) {
    throw std::runtime_error("Phase 8 collection count outside reviewed bounds");
  }
  std::unordered_set<std::string> identifiers;
  for (const auto& body : bodies) {
    require_members(body, {"body_id", "body_kind", "transform", "active"}, "body declaration");
    const auto body_id = id(member(body, "body_id", "body"), "body ID");
    if (!identifiers.insert(body_id).second) throw std::runtime_error("duplicate Phase 8 ID");
    static_cast<void>(body_kind(member(body, "body_kind", "body")));
    static_cast<void>(transform(member(body, "transform", "body")));
    static_cast<void>(boolean(member(body, "active", "body"), "body active"));
  }
  for (const auto& fixture : fixtures) {
    require_members(
        fixture,
        {"fixture_id", "owner_body_id", "shape", "density_bits", "friction_bits",
         "restitution_bits", "sensor", "filter"},
        "fixture declaration");
    const auto fixture_id = id(member(fixture, "fixture_id", "fixture"), "fixture ID");
    if (!identifiers.insert(fixture_id).second) throw std::runtime_error("duplicate Phase 8 ID");
    static_cast<void>(shape(member(fixture, "shape", "fixture")));
    static_cast<void>(filter(member(fixture, "filter", "fixture")));
    validate_phase8_bits(fixture, "Phase 8 fixture declaration");
  }
  for (const auto& joint : joints) {
    validate_phase8_joint(joint);
    if (!identifiers.insert(text(member(joint, "joint_id", "joint"), "joint ID")).second) {
      throw std::runtime_error("duplicate Phase 8 ID");
    }
  }
  for (const auto& rope : ropes) {
    validate_phase8_rope(rope);
    if (!identifiers.insert(text(member(rope, "rope_id", "rope"), "rope ID")).second) {
      throw std::runtime_error("duplicate Phase 8 ID");
    }
  }
  std::unordered_set<std::string> action_ids;
  for (const auto& record : actions) {
    require_members(record, {"action_id", "phase", "action"}, "Phase 8 action record");
    if (!action_ids.insert(id(member(record, "action_id", "action"), "action ID")).second) {
      throw std::runtime_error("duplicate Phase 8 action ID");
    }
    const auto& action = member(record, "action", "action record");
    const auto kind = text(member(action, "kind", "action"), "action kind");
    if (!phase8_known_action(kind)) throw std::runtime_error("unsupported Phase 8 action kind");
    validate_phase8_bits(action, "Phase 8 action");
  }
  for (const auto& checkpoint : checkpoints) {
    require_members(
        checkpoint,
        {"checkpoint_id", "after_action_id", "phase", "counts", "transitions"},
        "Phase 8 checkpoint");
    static_cast<void>(id(member(checkpoint, "checkpoint_id", "checkpoint"), "checkpoint ID"));
    const auto after = id(member(checkpoint, "after_action_id", "checkpoint"), "after-action ID");
    if (!action_ids.count(after)) throw std::runtime_error("invalid Phase 8 checkpoint action");
    const auto& transitions = member(checkpoint, "transitions", "checkpoint");
    if (!transitions.is_array() || transitions.empty() || transitions.size() > 64) {
      throw std::runtime_error("Phase 8 transition count outside reviewed bounds");
    }
  }
}

inline std::vector<std::string> decode_phase8_timelines(const Json& timelines) {
  if (timelines.size() != 19) {
    throw std::runtime_error("Phase 8 timeline count outside reviewed bounds");
  }
  std::vector<std::string> result;
  result.reserve(phase8_family_names().size());
  for (std::size_t index = 0; index < phase8_family_names().size(); ++index) {
    const auto& timeline = timelines.at(index + 9);
    validate_phase8_timeline(timeline, phase8_family_names().at(index));
    result.push_back(timeline.dump());
  }
  return result;
}
