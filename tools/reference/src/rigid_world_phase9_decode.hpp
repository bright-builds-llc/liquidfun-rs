#pragma once

// Included by rigid_world_decode.hpp inside
// liquidfun::reference::rigid_world_decode after the shared JSON helpers.

inline constexpr std::size_t kPhase9MaximumSystems = 16;
inline constexpr std::size_t kPhase9MaximumParticles = 256;
inline constexpr std::size_t kPhase9MaximumIdentities = 256;

inline bool phase9_particle_action(const Json& record) {
  const auto found = record.find("action");
  return found != record.end() && found->is_object() &&
         found->value("kind", std::string{}) == "particle";
}

inline void validate_phase9_capacity(
    const Json& value,
    std::string_view name) {
  const auto capacity = unsigned_value(value, name);
  if (capacity == 0 || capacity > kPhase9MaximumParticles) {
    throw std::runtime_error(
        std::string(name) + " is outside reviewed Phase 9 bounds");
  }
}

inline void validate_phase9_system(const Json& system) {
  require_members(
      system,
      {"system_id", "buffer_mode", "paused", "strict_contact_check",
       "stuck_threshold", "density_bits", "gravity_scale_bits", "radius_bits",
       "damping_bits", "destruction_by_age", "lifetime_granularity_bits",
       "maximum_count"},
      "Phase 9 particle-system declaration");
  static_cast<void>(id(member(system, "system_id", "particle system"), "system ID"));
  const auto& buffer = member(system, "buffer_mode", "particle system");
  const auto buffer_kind =
      text(member(buffer, "kind", "buffer mode"), "buffer kind");
  if (buffer_kind == "growable") {
    require_members(buffer, {"kind", "initial_capacity"}, "growable buffer mode");
    validate_phase9_capacity(
        member(buffer, "initial_capacity", "buffer mode"), "initial capacity");
  } else if (buffer_kind == "fixed") {
    require_members(buffer, {"kind", "capacity"}, "fixed buffer mode");
    validate_phase9_capacity(
        member(buffer, "capacity", "buffer mode"), "fixed capacity");
  } else {
    throw std::runtime_error("unsupported Phase 9 buffer mode");
  }
  static_cast<void>(boolean(member(system, "paused", "particle system"), "paused"));
  static_cast<void>(boolean(
      member(system, "strict_contact_check", "particle system"),
      "strict contact check"));
  static_cast<void>(u32(
      member(system, "stuck_threshold", "particle system"),
      "stuck threshold"));
  for (const auto* name : {"density_bits", "radius_bits",
                           "lifetime_granularity_bits"}) {
    const auto bits = u32(member(system, name, "particle system"), name);
    require_finite(bits, name);
    if (float_from_bits(bits) <= 0.0F) {
      throw std::runtime_error(std::string(name) + " must be positive");
    }
  }
  const auto gravity = u32(
      member(system, "gravity_scale_bits", "particle system"),
      "gravity scale bits");
  require_finite(gravity, "gravity scale bits");
  const auto damping = u32(
      member(system, "damping_bits", "particle system"), "damping bits");
  require_nonnegative(damping, "damping bits");
  static_cast<void>(boolean(
      member(system, "destruction_by_age", "particle system"),
      "destruction by age"));
  const auto& maximum = member(system, "maximum_count", "particle system");
  if (!maximum.is_null()) validate_phase9_capacity(maximum, "maximum count");
}

inline void validate_phase9_particle(
    const Json& particle,
    const std::set<std::string>& system_ids) {
  require_members(
      particle,
      {"particle_id", "system_id", "position", "velocity", "flags_bits",
       "color", "lifetime_bits"},
      "Phase 9 particle declaration");
  static_cast<void>(id(member(particle, "particle_id", "particle"), "particle ID"));
  const auto owner = id(member(particle, "system_id", "particle"), "system ID");
  if (!system_ids.count(owner)) {
    throw std::runtime_error("Phase 9 particle has unknown system owner");
  }
  static_cast<void>(vec2(member(particle, "position", "particle"), "particle position"));
  static_cast<void>(vec2(member(particle, "velocity", "particle"), "particle velocity"));
  const auto flags = u32(member(particle, "flags_bits", "particle"), "particle flags");
  constexpr std::uint32_t kKnownFlags = (1U << 18U) - 1U;
  constexpr std::uint32_t kPairOrTriadFlags =
      (1U << 3U) | (1U << 4U) | (1U << 10U) | (1U << 12U);
  if ((flags & ~kKnownFlags) != 0U || (flags & kPairOrTriadFlags) != 0U) {
    throw std::runtime_error(
        "unsupported Phase 9 particle flags or deferred pair/triad generation");
  }
  const auto& color = member(particle, "color", "particle");
  if (!color.is_array() || color.size() != 4) {
    throw std::runtime_error("Phase 9 particle color must contain four bytes");
  }
  for (const auto& component : color) {
    if (unsigned_value(component, "particle color") > 255U) {
      throw std::runtime_error("Phase 9 particle color exceeds u8");
    }
  }
  const auto lifetime =
      u32(member(particle, "lifetime_bits", "particle"), "lifetime bits");
  require_finite(lifetime, "lifetime bits");
}

struct Phase9ValidationState {
  std::set<std::string> created_systems;
  std::set<std::string> live_systems;
  std::set<std::string> created_particles;
  std::set<std::string> live_particles;
  std::set<std::string> pending_particles;
};

inline std::vector<std::string> phase9_id_list(
    const Json& value,
    std::string_view context) {
  if (!value.is_array() || value.empty() ||
      value.size() > kPhase9MaximumIdentities) {
    throw std::runtime_error(
        std::string(context) + " identity count outside reviewed bounds");
  }
  std::vector<std::string> result;
  std::set<std::string> unique;
  for (const auto& raw : value) {
    auto value_id = id(raw, context);
    if (!unique.insert(value_id).second) {
      throw std::runtime_error(std::string(context) + " contains duplicate ID");
    }
    result.push_back(std::move(value_id));
  }
  return result;
}

inline void validate_phase9_action(
    const Json& action,
    const std::set<std::string>& system_ids,
    const std::set<std::string>& particle_ids) {
  const auto kind = text(member(action, "kind", "Phase 9 action"), "action kind");
  const auto require_system = [&](std::string_view context) {
    const auto value = id(member(action, "system_id", context), "system ID");
    if (!system_ids.count(value)) {
      throw std::runtime_error("Phase 9 action references unknown system");
    }
  };
  const auto require_particle = [&](std::string_view context) {
    const auto value = id(member(action, "particle_id", context), "particle ID");
    if (!particle_ids.count(value)) {
      throw std::runtime_error("Phase 9 action references unknown particle");
    }
  };
  if (kind == "create_system" || kind == "destroy_system" ||
      kind == "inspect_system" || kind == "compact" ||
      kind == "request_statistics") {
    require_members(action, {"kind", "system_id"}, "Phase 9 system action");
    return require_system("Phase 9 system action");
  }
  if (kind == "inspect_particle_contact" || kind == "inspect_body_contact") {
    require_members(
        action, {"kind", "system_id", "contact_index"},
        "Phase 9 contact-inspection action");
    require_system("Phase 9 contact-inspection action");
    if (unsigned_value(
            member(action, "contact_index", "Phase 9 contact-inspection action"),
            "contact index") >= kPhase9MaximumIdentities) {
      throw std::runtime_error("Phase 9 contact index exceeds reviewed bounds");
    }
    return;
  }
  if (kind == "create_particle" || kind == "inspect_particle" ||
      kind == "mark_for_destruction") {
    require_members(action, {"kind", "particle_id"}, "Phase 9 particle action");
    return require_particle("Phase 9 particle action");
  }
  if (kind == "set_paused") {
    require_members(action, {"kind", "system_id", "paused"}, "set-paused action");
    require_system("set-paused action");
    static_cast<void>(boolean(member(action, "paused", "set-paused action"), "paused"));
    return;
  }
  if (kind == "set_position" || kind == "set_velocity") {
    const auto vector_name = kind == "set_position" ? "position" : "velocity";
    require_members(action, {"kind", "particle_id", vector_name}, "particle-vector action");
    require_particle("particle-vector action");
    static_cast<void>(vec2(member(action, vector_name, "particle-vector action"), vector_name));
    return;
  }
  if (kind == "apply_force" || kind == "apply_impulse") {
    const auto vector_name = kind == "apply_force" ? "force" : "impulse";
    require_members(action, {"kind", "particle_ids", vector_name}, "particle-range action");
    for (const auto& particle_id : phase9_id_list(
             member(action, "particle_ids", "particle-range action"),
             "particle range")) {
      if (!particle_ids.count(particle_id)) {
        throw std::runtime_error("Phase 9 range references unknown particle");
      }
    }
    static_cast<void>(vec2(member(action, vector_name, "particle-range action"), vector_name));
    return;
  }
  if (kind == "query_aabb") {
    require_members(action, {"kind", "system_id", "lower", "upper", "control"}, "particle query");
    const auto& maybe_system = member(action, "system_id", "particle query");
    if (!maybe_system.is_null()) require_system("particle query");
    const auto lower = vec2(member(action, "lower", "particle query"), "query lower");
    const auto upper = vec2(member(action, "upper", "particle query"), "query upper");
    if (float_from_bits(lower.x) > float_from_bits(upper.x) ||
        float_from_bits(lower.y) > float_from_bits(upper.y)) {
      throw std::runtime_error("Phase 9 query AABB is invalid");
    }
    const auto control = action.value("control", std::string{"continue"});
    if (control != "continue" && control != "terminate") {
      throw std::runtime_error("unsupported Phase 9 query control");
    }
    return;
  }
  if (kind == "ray_cast") {
    require_members(action, {"kind", "system_id", "start", "end", "control"}, "particle ray");
    const auto& maybe_system = member(action, "system_id", "particle ray");
    if (!maybe_system.is_null()) require_system("particle ray");
    const auto start = vec2(member(action, "start", "particle ray"), "ray start");
    const auto end = vec2(member(action, "end", "particle ray"), "ray end");
    if (start.x == end.x && start.y == end.y) {
      throw std::runtime_error("Phase 9 ray must be nondegenerate");
    }
    const auto control = action.value("control", std::string{"continue"});
    if (control != "ignore" && control != "continue" && control != "clip" &&
        control != "terminate") {
      throw std::runtime_error("unsupported Phase 9 ray control");
    }
    return;
  }
  throw std::runtime_error("unsupported Phase 9 or Phase 10 particle action");
}

inline void validate_phase9_action_lifecycle(
    const Json& action,
    const std::set<std::string>& system_ids,
    const std::unordered_map<std::string, std::string>& particle_owners,
    Phase9ValidationState& state) {
  const auto kind = action.at("kind").get<std::string>();
  const auto system_id = [&]() {
    const auto value = action.at("system_id").get<std::string>();
    if (!system_ids.count(value)) {
      throw std::runtime_error("Phase 9 action references unknown system");
    }
    return value;
  };
  const auto particle_id = [&]() {
    const auto value = action.at("particle_id").get<std::string>();
    if (!particle_owners.count(value)) {
      throw std::runtime_error("Phase 9 action references unknown particle");
    }
    return value;
  };
  const auto require_live_system = [&](const std::string& value) {
    if (!state.live_systems.count(value)) {
      throw std::runtime_error("Phase 9 particle system is not live");
    }
  };
  const auto require_live_particle = [&](const std::string& value) {
    const auto& owner = particle_owners.at(value);
    if (!state.live_systems.count(owner) || !state.live_particles.count(value)) {
      throw std::runtime_error("Phase 9 particle is not live");
    }
    return owner;
  };

  if (kind == "create_system") {
    const auto value = system_id();
    if (!state.created_systems.insert(value).second ||
        !state.live_systems.insert(value).second) {
      throw std::runtime_error("duplicate Phase 9 system creation");
    }
    return;
  }
  if (kind == "destroy_system") {
    const auto value = system_id();
    require_live_system(value);
    state.live_systems.erase(value);
    for (const auto& [particle, owner] : particle_owners) {
      if (owner == value) {
        state.live_particles.erase(particle);
        state.pending_particles.erase(particle);
      }
    }
    return;
  }
  if (kind == "create_particle") {
    const auto value = particle_id();
    require_live_system(particle_owners.at(value));
    if (!state.created_particles.insert(value).second ||
        !state.live_particles.insert(value).second) {
      throw std::runtime_error("duplicate Phase 9 particle creation");
    }
    return;
  }
  if (kind == "inspect_system" || kind == "set_paused" ||
      kind == "request_statistics" || kind == "inspect_particle_contact" ||
      kind == "inspect_body_contact") {
    require_live_system(system_id());
    return;
  }
  if (kind == "compact") {
    const auto value = system_id();
    require_live_system(value);
    for (const auto& [particle, owner] : particle_owners) {
      if (owner == value) state.pending_particles.erase(particle);
    }
    return;
  }
  if (kind == "inspect_particle" || kind == "set_position" ||
      kind == "set_velocity") {
    static_cast<void>(require_live_particle(particle_id()));
    return;
  }
  if (kind == "mark_for_destruction") {
    const auto value = particle_id();
    static_cast<void>(require_live_particle(value));
    state.live_particles.erase(value);
    state.pending_particles.insert(value);
    return;
  }
  if (kind == "apply_force" || kind == "apply_impulse") {
    std::string owner;
    for (const auto& raw : action.at("particle_ids")) {
      const auto value = raw.get<std::string>();
      const auto& candidate_owner = require_live_particle(value);
      if (!owner.empty() && owner != candidate_owner) {
        throw std::runtime_error("Phase 9 range crosses particle systems");
      }
      owner = candidate_owner;
    }
    return;
  }
  if (kind == "query_aabb" || kind == "ray_cast") {
    if (!action.at("system_id").is_null()) require_live_system(system_id());
    return;
  }
  throw std::runtime_error("unsupported Phase 9 lifecycle action");
}

inline bool validate_phase9_timeline(const Json& timeline) {
  const auto systems_it = timeline.find("particle_systems");
  const auto particles_it = timeline.find("particles");
  bool has_particle_action = false;
  for (const auto& record : timeline.at("actions")) {
    has_particle_action = has_particle_action || phase9_particle_action(record);
  }
  if (systems_it == timeline.end() && particles_it == timeline.end() &&
      !has_particle_action) {
    return false;
  }
  if (systems_it == timeline.end() || particles_it == timeline.end() ||
      !systems_it->is_array() || !particles_it->is_array() ||
      systems_it->size() > kPhase9MaximumSystems ||
      particles_it->size() > kPhase9MaximumParticles) {
    throw std::runtime_error("Phase 9 declarations are missing or exceed bounds");
  }
  std::set<std::string> system_ids;
  for (const auto& system : *systems_it) {
    validate_phase9_system(system);
    if (!system_ids.insert(system.at("system_id").get<std::string>()).second) {
      throw std::runtime_error("duplicate Phase 9 system ID");
    }
  }
  std::set<std::string> particle_ids;
  std::unordered_map<std::string, std::string> particle_owners;
  for (const auto& particle : *particles_it) {
    validate_phase9_particle(particle, system_ids);
    if (!particle_ids.insert(particle.at("particle_id").get<std::string>()).second) {
      throw std::runtime_error("duplicate Phase 9 particle ID");
    }
    particle_owners.emplace(
        particle.at("particle_id").get<std::string>(),
        particle.at("system_id").get<std::string>());
  }
  Phase9ValidationState state;
  for (const auto& record : timeline.at("actions")) {
    if (!phase9_particle_action(record)) continue;
    require_members(record, {"action_id", "phase", "action"}, "Phase 9 action record");
    const auto& wrapper = record.at("action");
    require_members(wrapper, {"kind", "action"}, "Phase 9 action wrapper");
    if (wrapper.at("kind") != "particle" || record.at("phase") != "phase9") {
      throw std::runtime_error("invalid Phase 9 action declaration family");
    }
    validate_phase9_action(wrapper.at("action"), system_ids, particle_ids);
    validate_phase9_action_lifecycle(
        wrapper.at("action"), system_ids, particle_owners, state);
  }
  if (!state.live_systems.empty() || !state.live_particles.empty() ||
      !state.pending_particles.empty()) {
    throw std::runtime_error("Phase 9 timeline leaves particle state live");
  }
  return true;
}

inline std::vector<std::string> decode_phase9_timelines(const Json& timelines) {
  std::vector<std::string> result(timelines.size());
  for (std::size_t index = 0; index < timelines.size(); ++index) {
    if (validate_phase9_timeline(timelines.at(index))) {
      result.at(index) = timelines.at(index).dump();
    }
  }
  return result;
}

inline Json strip_phase9_for_legacy_decode(Json root) {
  for (auto& timeline : root["scenario"]["timelines"]) {
    timeline.erase("particle_systems");
    timeline.erase("particles");
    auto original_actions = timeline.at("actions");
    Json retained_actions = Json::array();
    std::unordered_map<std::string, std::string> checkpoint_predecessors;
    std::unordered_map<std::string, std::string> predecessor_phases;
    std::string maybe_previous;
    std::string maybe_previous_phase;
    for (const auto& record : original_actions) {
      const auto action_id = record.at("action_id").get<std::string>();
      if (!phase9_particle_action(record)) {
        retained_actions.push_back(record);
        maybe_previous = action_id;
        maybe_previous_phase = record.at("phase").get<std::string>();
      }
      checkpoint_predecessors.emplace(action_id, maybe_previous);
      predecessor_phases.emplace(action_id, maybe_previous_phase);
    }
    Json retained_checkpoints = Json::array();
    for (auto checkpoint : timeline.at("checkpoints")) {
      if (checkpoint.at("phase") == "phase9") continue;
      const auto after = checkpoint.at("after_action_id").get<std::string>();
      const auto found = checkpoint_predecessors.find(after);
      if (found == checkpoint_predecessors.end() || found->second.empty()) {
        continue;
      }
      checkpoint["after_action_id"] = found->second;
      checkpoint["phase"] = predecessor_phases.at(after);
      retained_checkpoints.push_back(std::move(checkpoint));
    }
    timeline["actions"] = std::move(retained_actions);
    timeline["checkpoints"] = std::move(retained_checkpoints);
  }
  return root;
}
