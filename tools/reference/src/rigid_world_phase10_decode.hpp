#pragma once

// Included by rigid_world_decode.hpp inside
// liquidfun::reference::rigid_world_decode after the shared JSON helpers.

inline constexpr std::size_t kPhase10MaximumOperations = 128;
inline constexpr std::size_t kPhase10MaximumGroups = 64;
inline constexpr std::size_t kPhase10MaximumParticles = 512;
inline constexpr std::size_t kPhase10MaximumShapes = 32;
inline constexpr std::size_t kPhase10MaximumShapeVertices = 64;
inline constexpr std::uint32_t kPhase10MaximumSteps = 1024;
inline constexpr std::uint32_t kPhase10ParticleFlags = 0x0003fffeU;
inline constexpr std::uint32_t kPhase10GroupFlags = 0x00000007U;

inline bool phase10_group_action(const Json& record) {
  const auto found = record.find("action");
  return found != record.end() && found->is_object() &&
         found->value("kind", std::string{}) == "particle_group";
}

inline void validate_phase10_positive_bits(
    const Json& value,
    std::string_view context) {
  const auto bits = u32(value, context);
  require_finite(bits, context);
  if (float_from_bits(bits) <= 0.0F) {
    throw std::runtime_error(std::string(context) + " must be positive");
  }
}

inline std::vector<std::string> phase10_ids(
    const Json& value,
    std::size_t minimum,
    std::size_t maximum,
    std::string_view context) {
  if (!value.is_array() || value.size() < minimum || value.size() > maximum) {
    throw std::runtime_error(
        std::string(context) + " identity count is outside reviewed bounds");
  }
  std::vector<std::string> result;
  result.reserve(value.size());
  std::set<std::string> unique;
  for (const auto& raw : value) {
    auto semantic_id = id(raw, context);
    if (!unique.insert(semantic_id).second) {
      throw std::runtime_error(std::string(context) + " contains duplicate ID");
    }
    result.push_back(std::move(semantic_id));
  }
  return result;
}

inline void validate_phase10_shape(const Json& shape) {
  const auto kind = text(member(shape, "kind", "Phase 10 shape"), "shape kind");
  if (kind == "circle") {
    require_members(shape, {"kind", "center", "radius_bits"}, "Phase 10 circle");
    static_cast<void>(vec2(member(shape, "center", "circle"), "circle center"));
    validate_phase10_positive_bits(
        member(shape, "radius_bits", "circle"), "circle radius bits");
    return;
  }
  if (kind == "polygon") {
    require_members(shape, {"kind", "vertices"}, "Phase 10 polygon");
    const auto& vertices = member(shape, "vertices", "polygon");
    if (!vertices.is_array() || vertices.size() < 3 || vertices.size() > 8) {
      throw std::runtime_error("Phase 10 polygon vertex count is outside reviewed bounds");
    }
    for (const auto& vertex : vertices) {
      static_cast<void>(vec2(vertex, "polygon vertex"));
    }
    return;
  }
  if (kind == "edge") {
    require_members(shape, {"kind", "vertex_a", "vertex_b"}, "Phase 10 edge");
    const auto a = vec2(member(shape, "vertex_a", "edge"), "edge vertex A");
    const auto b = vec2(member(shape, "vertex_b", "edge"), "edge vertex B");
    if (a.x == b.x && a.y == b.y) {
      throw std::runtime_error("Phase 10 edge must be nondegenerate");
    }
    return;
  }
  if (kind == "chain") {
    require_members(shape, {"kind", "vertices", "looped"}, "Phase 10 chain");
    const auto looped = boolean(member(shape, "looped", "chain"), "chain looped");
    const auto& vertices = member(shape, "vertices", "chain");
    const auto minimum = looped ? std::size_t{3} : std::size_t{2};
    if (!vertices.is_array() || vertices.size() < minimum ||
        vertices.size() > kPhase10MaximumShapeVertices) {
      throw std::runtime_error("Phase 10 chain vertex count is outside reviewed bounds");
    }
    for (const auto& vertex : vertices) {
      static_cast<void>(vec2(vertex, "chain vertex"));
    }
    return;
  }
  throw std::runtime_error("unsupported Phase 10 shape");
}

inline void validate_phase10_provenance(const Json& provenance) {
  require_members(
      provenance,
      {"extension_version", "generator_id", "generator_version",
       "upstream_revision", "toolchain_id", "seed"},
      "Phase 10 provenance");
  if (u32(member(provenance, "extension_version", "provenance"),
          "extension version") != 1U) {
    throw std::runtime_error("unsupported Phase 10 extension version");
  }
  for (const auto* name : {"generator_id", "generator_version",
                           "upstream_revision", "toolchain_id"}) {
    static_cast<void>(id(member(provenance, name, "provenance"), name));
  }
  if (provenance.at("upstream_revision") !=
      "7f20402173fd143a3988c921bc384459c6a858f2") {
    throw std::runtime_error("Phase 10 provenance does not name the pinned revision");
  }
  static_cast<void>(unsigned_value(member(provenance, "seed", "provenance"), "seed"));
}

inline void validate_phase10_definition(
    const Json& definition,
    const std::set<std::string>& system_ids) {
  require_members(
      definition,
      {"provenance", "system_id", "group_id", "member_ids", "source",
       "destination", "particle_flags_bits", "group_flags_bits", "transform",
       "linear_velocity", "angular_velocity_bits", "color", "strength_bits",
       "maybe_stride_bits", "lifetime_bits"},
      "Phase 10 group definition");
  validate_phase10_provenance(member(definition, "provenance", "group definition"));
  const auto system_id =
      id(member(definition, "system_id", "group definition"), "system ID");
  if (!system_ids.count(system_id)) {
    throw std::runtime_error("Phase 10 group references unknown particle system");
  }
  static_cast<void>(id(member(definition, "group_id", "group definition"), "group ID"));
  const auto member_ids = phase10_ids(
      member(definition, "member_ids", "group definition"), 1,
      kPhase10MaximumParticles, "group member IDs");
  const auto& source = member(definition, "source", "group definition");
  const auto source_kind = text(member(source, "kind", "group source"), "source kind");
  std::size_t source_count = member_ids.size();
  if (source_kind == "filled") {
    require_members(source, {"kind", "shapes"}, "filled group source");
    const auto& shapes = member(source, "shapes", "filled source");
    if (!shapes.is_array() || shapes.empty() ||
        shapes.size() > kPhase10MaximumShapes) {
      throw std::runtime_error("Phase 10 filled shape count is outside reviewed bounds");
    }
    for (const auto& shape : shapes) {
      validate_phase10_shape(shape);
      const auto kind = shape.at("kind").get<std::string>();
      if (kind != "circle" && kind != "polygon") {
        throw std::runtime_error("Phase 10 filled source requires area shapes");
      }
    }
  } else if (source_kind == "stroke") {
    require_members(source, {"kind", "shape"}, "stroke group source");
    const auto& shape = member(source, "shape", "stroke source");
    validate_phase10_shape(shape);
    const auto kind = shape.at("kind").get<std::string>();
    if (kind != "edge" && kind != "chain") {
      throw std::runtime_error("Phase 10 stroke source requires edge or chain");
    }
  } else if (source_kind == "explicit") {
    require_members(source, {"kind", "positions"}, "explicit group source");
    const auto& positions = member(source, "positions", "explicit source");
    if (!positions.is_array() || positions.empty() ||
        positions.size() > kPhase10MaximumParticles) {
      throw std::runtime_error("Phase 10 explicit positions exceed reviewed bounds");
    }
    source_count = positions.size();
    for (const auto& position : positions) {
      static_cast<void>(vec2(position, "explicit position"));
    }
  } else {
    throw std::runtime_error("unsupported Phase 10 group source");
  }
  if (source_count != member_ids.size()) {
    throw std::runtime_error("Phase 10 explicit source and member IDs are misaligned");
  }
  const auto& destination = member(definition, "destination", "group definition");
  const auto destination_kind =
      text(member(destination, "kind", "group destination"), "destination kind");
  if (destination_kind == "new") {
    require_members(destination, {"kind"}, "new group destination");
  } else if (destination_kind == "append_to") {
    require_members(destination, {"kind", "target_group_id"}, "append destination");
    const auto target = id(
        member(destination, "target_group_id", "append destination"),
        "append target group ID");
    if (target != definition.at("group_id").get<std::string>()) {
      throw std::runtime_error("Phase 10 append identity differs from its target");
    }
  } else {
    throw std::runtime_error("unsupported Phase 10 group destination");
  }
  const auto particle_flags = u32(
      member(definition, "particle_flags_bits", "group definition"),
      "particle flags");
  const auto group_flags = u32(
      member(definition, "group_flags_bits", "group definition"), "group flags");
  if ((particle_flags & ~kPhase10ParticleFlags) != 0U ||
      (group_flags & ~kPhase10GroupFlags) != 0U) {
    throw std::runtime_error("Phase 10 flags contain private or unknown bits");
  }
  static_cast<void>(transform(member(definition, "transform", "group definition")));
  static_cast<void>(vec2(
      member(definition, "linear_velocity", "group definition"),
      "group linear velocity"));
  const auto angular = u32(
      member(definition, "angular_velocity_bits", "group definition"),
      "angular velocity bits");
  require_finite(angular, "angular velocity bits");
  const auto& color = member(definition, "color", "group definition");
  if (!color.is_array() || color.size() != 4) {
    throw std::runtime_error("Phase 10 color must contain four bytes");
  }
  for (const auto& component : color) {
    if (unsigned_value(component, "color component") > 255U) {
      throw std::runtime_error("Phase 10 color component exceeds u8");
    }
  }
  const auto strength = u32(
      member(definition, "strength_bits", "group definition"), "strength bits");
  require_nonnegative(strength, "strength bits");
  const auto& stride = member(definition, "maybe_stride_bits", "group definition");
  if (!stride.is_null()) validate_phase10_positive_bits(stride, "stride bits");
  const auto lifetime = u32(
      member(definition, "lifetime_bits", "group definition"), "lifetime bits");
  require_finite(lifetime, "lifetime bits");
}

struct Phase10ValidationState {
  std::set<std::string> live_systems;
  std::map<std::string, std::string> live_groups;
  std::set<std::string> all_groups;
  std::set<std::string> all_particles;
  std::string provenance;
  std::size_t operation_count = 0;
  std::uint32_t step_count = 0;
};

inline void validate_phase10_operation(
    const Json& operation,
    const std::set<std::string>& system_ids,
    Phase10ValidationState& state) {
  if (++state.operation_count > kPhase10MaximumOperations) {
    throw std::runtime_error("Phase 10 operation count exceeds reviewed bounds");
  }
  const auto kind = text(member(operation, "kind", "Phase 10 operation"), "operation kind");
  if (kind == "create_group") {
    require_members(operation, {"kind", "definition"}, "create-group operation");
    const auto& definition = member(operation, "definition", "create-group operation");
    validate_phase10_definition(definition, system_ids);
    const auto system_id = definition.at("system_id").get<std::string>();
    if (!state.live_systems.count(system_id)) {
      throw std::runtime_error("Phase 10 group owner is not live");
    }
    const auto encoded_provenance = definition.at("provenance").dump();
    if (state.provenance.empty()) state.provenance = encoded_provenance;
    else if (state.provenance != encoded_provenance) {
      throw std::runtime_error("Phase 10 provenance changed within one timeline");
    }
    for (const auto& raw : definition.at("member_ids")) {
      const auto particle_id = raw.get<std::string>();
      if (state.all_groups.count(particle_id) ||
          !state.all_particles.insert(particle_id).second) {
        throw std::runtime_error("duplicate Phase 10 particle semantic ID");
      }
    }
    const auto group_id = definition.at("group_id").get<std::string>();
    const auto destination = definition.at("destination").at("kind").get<std::string>();
    if (destination == "new") {
      if (state.all_particles.count(group_id) || !state.all_groups.insert(group_id).second) {
        throw std::runtime_error("duplicate Phase 10 group semantic ID");
      }
      state.live_groups.emplace(group_id, system_id);
    } else {
      const auto found = state.live_groups.find(group_id);
      if (found == state.live_groups.end()) {
        throw std::runtime_error("Phase 10 append target is not live");
      }
      if (found->second != system_id) {
        throw std::runtime_error("Phase 10 append crosses particle systems");
      }
    }
    if (state.all_groups.size() > kPhase10MaximumGroups ||
        state.all_particles.size() > kPhase10MaximumParticles) {
      throw std::runtime_error("Phase 10 semantic identities exceed reviewed bounds");
    }
    return;
  }
  if (kind == "join_groups") {
    require_members(operation, {"kind", "target_group_id", "source_group_id"}, "join-groups operation");
    const auto target = id(operation.at("target_group_id"), "target group ID");
    const auto source = id(operation.at("source_group_id"), "source group ID");
    if (target == source || !state.live_groups.count(target) ||
        !state.live_groups.count(source) ||
        state.live_groups.at(target) != state.live_groups.at(source)) {
      throw std::runtime_error("invalid Phase 10 group join");
    }
    state.live_groups.erase(source);
    return;
  }
  if (kind == "split_group") {
    require_members(operation, {"kind", "group_id", "created_group_ids"}, "split-group operation");
    const auto group_id = id(operation.at("group_id"), "split group ID");
    const auto found = state.live_groups.find(group_id);
    if (found == state.live_groups.end()) {
      throw std::runtime_error("Phase 10 split target is not live");
    }
    const auto created = phase10_ids(
        operation.at("created_group_ids"), 1, kPhase10MaximumGroups,
        "split-created group IDs");
    for (const auto& created_id : created) {
      if (state.all_particles.count(created_id) ||
          !state.all_groups.insert(created_id).second) {
        throw std::runtime_error("duplicate Phase 10 split group ID");
      }
      state.live_groups.emplace(created_id, found->second);
    }
    if (state.all_groups.size() > kPhase10MaximumGroups) {
      throw std::runtime_error("Phase 10 group identities exceed reviewed bounds");
    }
    return;
  }
  if (kind == "set_group_flags") {
    require_members(operation, {"kind", "group_id", "group_flags_bits"}, "set-group-flags operation");
    const auto group_id = id(operation.at("group_id"), "group ID");
    if (!state.live_groups.count(group_id) ||
        (u32(operation.at("group_flags_bits"), "group flags") &
         ~kPhase10GroupFlags) != 0U) {
      throw std::runtime_error("invalid Phase 10 group flags operation");
    }
    return;
  }
  if (kind == "destroy_group") {
    require_members(operation, {"kind", "group_id"}, "destroy-group operation");
    const auto group_id = id(operation.at("group_id"), "group ID");
    if (state.live_groups.erase(group_id) == 0U) {
      throw std::runtime_error("Phase 10 destroy target is not live");
    }
    return;
  }
  if (kind == "step") {
    require_members(
        operation,
        {"kind", "timestep_bits", "velocity_iterations",
         "position_iterations", "particle_iterations"},
        "Phase 10 step operation");
    validate_phase10_positive_bits(operation.at("timestep_bits"), "timestep bits");
    for (const auto* name : {"velocity_iterations", "position_iterations",
                             "particle_iterations"}) {
      const auto value = u32(operation.at(name), name);
      if (value == 0U || value > kPhase10MaximumSteps) {
        throw std::runtime_error("Phase 10 solver iterations exceed reviewed bounds");
      }
    }
    if (++state.step_count > kPhase10MaximumSteps) {
      throw std::runtime_error("Phase 10 step count exceeds reviewed bounds");
    }
    return;
  }
  if (kind == "inspect_state") {
    require_members(operation, {"kind"}, "inspect-state operation");
    if (state.provenance.empty()) {
      throw std::runtime_error("Phase 10 inspection has no provenance");
    }
    return;
  }
  throw std::runtime_error("unsupported Phase 10 operation");
}

inline bool validate_phase10_timeline(const Json& timeline) {
  bool has_group_action = false;
  for (const auto& record : timeline.at("actions")) {
    has_group_action = has_group_action || phase10_group_action(record);
  }
  if (!has_group_action) return false;
  const auto systems_it = timeline.find("particle_systems");
  if (systems_it == timeline.end() || !systems_it->is_array()) {
    throw std::runtime_error("Phase 10 requires particle-system declarations");
  }
  std::set<std::string> system_ids;
  for (const auto& system : *systems_it) {
    validate_phase9_system(system);
    if (!system_ids.insert(system.at("system_id").get<std::string>()).second) {
      throw std::runtime_error("duplicate Phase 10 particle-system ID");
    }
  }
  Phase10ValidationState state;
  for (const auto& record : timeline.at("actions")) {
    const auto& wrapper = record.at("action");
    if (wrapper.at("kind") == "particle") {
      const auto& action = wrapper.at("action");
      const auto kind = action.at("kind").get<std::string>();
      if (kind == "create_system") {
        state.live_systems.insert(action.at("system_id").get<std::string>());
      } else if (kind == "destroy_system") {
        const auto system_id = action.at("system_id").get<std::string>();
        if (std::any_of(
                state.live_groups.begin(), state.live_groups.end(),
                [&](const auto& group) { return group.second == system_id; })) {
          throw std::runtime_error("Phase 10 destroys a system with live groups");
        }
        state.live_systems.erase(system_id);
      }
      continue;
    }
    if (!phase10_group_action(record)) continue;
    require_members(record, {"action_id", "phase", "action"}, "Phase 10 action record");
    require_members(wrapper, {"kind", "operation"}, "Phase 10 action wrapper");
    if (record.at("phase") != "phase10") {
      throw std::runtime_error("invalid Phase 10 action phase");
    }
    validate_phase10_operation(wrapper.at("operation"), system_ids, state);
  }
  if (!state.live_groups.empty()) {
    throw std::runtime_error("Phase 10 timeline leaves particle groups live");
  }
  return true;
}

inline std::vector<std::string> decode_phase10_timelines(const Json& timelines) {
  std::vector<std::string> result(timelines.size());
  for (std::size_t index = 0; index < timelines.size(); ++index) {
    if (validate_phase10_timeline(timelines.at(index))) {
      result.at(index) = timelines.at(index).dump();
    }
  }
  return result;
}

inline Json strip_phase10_for_phase9(Json root) {
  for (auto& timeline : root["scenario"]["timelines"]) {
    if (timeline.contains("particle_systems") && !timeline.contains("particles")) {
      timeline["particles"] = Json::array();
    }
    auto original_actions = timeline.at("actions");
    Json retained_actions = Json::array();
    std::unordered_map<std::string, std::string> predecessors;
    std::unordered_map<std::string, std::string> predecessor_phases;
    std::string maybe_previous;
    std::string maybe_previous_phase;
    for (const auto& record : original_actions) {
      const auto action_id = record.at("action_id").get<std::string>();
      if (!phase10_group_action(record)) {
        retained_actions.push_back(record);
        maybe_previous = action_id;
        maybe_previous_phase = record.at("phase").get<std::string>();
      }
      predecessors.emplace(action_id, maybe_previous);
      predecessor_phases.emplace(action_id, maybe_previous_phase);
    }
    Json retained_checkpoints = Json::array();
    for (auto checkpoint : timeline.at("checkpoints")) {
      const auto after = checkpoint.at("after_action_id").get<std::string>();
      const auto found = predecessors.find(after);
      if (found == predecessors.end() || found->second.empty()) continue;
      checkpoint["after_action_id"] = found->second;
      checkpoint["phase"] = predecessor_phases.at(after);
      retained_checkpoints.push_back(std::move(checkpoint));
    }
    timeline["actions"] = std::move(retained_actions);
    timeline["checkpoints"] = std::move(retained_checkpoints);
  }
  return root;
}
