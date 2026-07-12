#pragma once

#include "protocol.hpp"
#include "rigid_world.hpp"

#include "nlohmann/json.hpp"

#include <algorithm>
#include <array>
#include <cmath>
#include <limits>
#include <set>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <unordered_set>

namespace liquidfun::reference::rigid_world_decode {

using Json = nlohmann::json;

inline void require_members(
    const Json& object,
    std::initializer_list<std::string_view> allowed,
    std::string_view context) {
  if (!object.is_object()) {
    throw std::runtime_error(std::string(context) + " must be an object");
  }
  for (const auto& [name, value] : object.items()) {
    static_cast<void>(value);
    if (std::find(allowed.begin(), allowed.end(), name) == allowed.end()) {
      throw std::runtime_error(
          std::string(context) + " contains unknown member " + name);
    }
  }
}

inline const Json& member(
    const Json& object,
    std::string_view name,
    std::string_view context) {
  const auto found = object.find(name);
  if (found == object.end()) {
    throw std::runtime_error(
        std::string(context) + " is missing member " + std::string(name));
  }
  return *found;
}

inline std::uint64_t unsigned_value(
    const Json& value,
    std::string_view context) {
  if (!value.is_number_unsigned() && !value.is_number_integer()) {
    throw std::runtime_error(std::string(context) + " must be unsigned");
  }
  if (value.is_number_integer() && value.get<std::int64_t>() < 0) {
    throw std::runtime_error(std::string(context) + " must be unsigned");
  }
  return value.get<std::uint64_t>();
}

inline std::uint32_t u32(const Json& value, std::string_view context) {
  const auto result = unsigned_value(value, context);
  if (result > std::numeric_limits<std::uint32_t>::max()) {
    throw std::runtime_error(std::string(context) + " exceeds u32");
  }
  return static_cast<std::uint32_t>(result);
}

inline std::uint16_t u16(const Json& value, std::string_view context) {
  const auto result = unsigned_value(value, context);
  if (result > std::numeric_limits<std::uint16_t>::max()) {
    throw std::runtime_error(std::string(context) + " exceeds u16");
  }
  return static_cast<std::uint16_t>(result);
}

inline std::int16_t i16(const Json& value, std::string_view context) {
  if (!value.is_number_integer()) {
    throw std::runtime_error(std::string(context) + " must be an integer");
  }
  const auto result = value.get<std::int64_t>();
  if (result < std::numeric_limits<std::int16_t>::min() ||
      result > std::numeric_limits<std::int16_t>::max()) {
    throw std::runtime_error(std::string(context) + " exceeds i16");
  }
  return static_cast<std::int16_t>(result);
}

inline std::string text(const Json& value, std::string_view context) {
  if (!value.is_string()) {
    throw std::runtime_error(std::string(context) + " must be a string");
  }
  return value.get<std::string>();
}

inline bool boolean(const Json& value, std::string_view context) {
  if (!value.is_boolean()) {
    throw std::runtime_error(std::string(context) + " must be a boolean");
  }
  return value.get<bool>();
}

inline bool valid_id(std::string_view value) {
  if (value.empty() || value.size() > kMaximumIdBytes) return false;
  const auto first = [](unsigned char character) {
    return (character >= 'a' && character <= 'z') ||
           (character >= '0' && character <= '9');
  };
  const auto rest = [first](unsigned char character) {
    return first(character) || character == '.' || character == '_' ||
           character == '-';
  };
  return first(static_cast<unsigned char>(value.front())) &&
         std::all_of(value.begin() + 1, value.end(), rest);
}

inline std::string id(const Json& value, std::string_view context) {
  auto result = text(value, context);
  if (!valid_id(result)) {
    throw std::runtime_error(std::string(context) + " is not a valid ID");
  }
  return result;
}

inline void require_finite(std::uint32_t bits, std::string_view context) {
  if (!std::isfinite(float_from_bits(bits))) {
    throw std::runtime_error(std::string(context) + " must be finite");
  }
}

inline void require_nonnegative(
    std::uint32_t bits,
    std::string_view context) {
  require_finite(bits, context);
  if (float_from_bits(bits) < 0.0F) {
    throw std::runtime_error(std::string(context) + " must be nonnegative");
  }
}

inline RigidVec2Bits vec2(const Json& value, std::string_view context) {
  require_members(value, {"x_bits", "y_bits"}, context);
  RigidVec2Bits result{
      u32(member(value, "x_bits", context), "x bits"),
      u32(member(value, "y_bits", context), "y bits")};
  require_finite(result.x, "x bits");
  require_finite(result.y, "y bits");
  return result;
}

inline RigidTransformBits transform(const Json& value) {
  require_members(value, {"position", "angle_bits"}, "transform");
  RigidTransformBits result{
      vec2(member(value, "position", "transform"), "position"),
      u32(member(value, "angle_bits", "transform"), "angle bits")};
  require_finite(result.angle, "angle bits");
  return result;
}

inline RigidFilterBits filter(const Json& value) {
  require_members(
      value, {"category_bits", "mask_bits", "group_index"}, "filter");
  return {
      u16(member(value, "category_bits", "filter"), "category bits"),
      u16(member(value, "mask_bits", "filter"), "mask bits"),
      i16(member(value, "group_index", "filter"), "group index")};
}

inline RigidBodyKind body_kind(const Json& value) {
  const auto name = text(value, "body kind");
  if (name == "static") return RigidBodyKind::static_body;
  if (name == "kinematic") return RigidBodyKind::kinematic_body;
  if (name == "dynamic") return RigidBodyKind::dynamic_body;
  throw std::runtime_error("unsupported body kind");
}

inline RigidShape shape(const Json& value) {
  const auto kind = text(member(value, "kind", "shape"), "shape kind");
  if (kind == "circle") {
    require_members(value, {"kind", "center", "radius_bits"}, "circle");
    const auto radius =
        u32(member(value, "radius_bits", "circle"), "circle radius bits");
    require_finite(radius, "circle radius bits");
    if (float_from_bits(radius) <= 0.0F) {
      throw std::runtime_error("circle radius must be positive");
    }
    return RigidCircleShape{
        vec2(member(value, "center", "circle"), "circle center"), radius};
  }
  if (kind == "polygon") {
    require_members(value, {"kind", "vertices"}, "polygon");
    const auto& vertices = member(value, "vertices", "polygon");
    if (!vertices.is_array() || vertices.size() < 3 || vertices.size() > 8) {
      throw std::runtime_error("polygon vertex count is outside reviewed bounds");
    }
    RigidPolygonShape result;
    for (const auto& vertex : vertices) {
      result.vertices.push_back(vec2(vertex, "polygon vertex"));
    }
    return result;
  }
  throw std::runtime_error("unsupported fixture shape");
}

inline RigidAction action(const Json& value) {
  const auto kind = text(member(value, "kind", "action"), "action kind");
  if (kind == "create_body") {
    require_members(value, {"kind", "body_id"}, "create-body action");
    return CreateBody{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "create_fixture") {
    require_members(value, {"kind", "fixture_id"}, "create-fixture action");
    return CreateFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "inspect_body") {
    require_members(value, {"kind", "body_id"}, "inspect-body action");
    return InspectBody{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "inspect_fixture") {
    require_members(value, {"kind", "fixture_id"}, "inspect-fixture action");
    return InspectFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "set_body_transform") {
    require_members(value, {"kind", "body_id", "transform"}, "body-transform action");
    return SetBodyTransform{
        id(member(value, "body_id", "action"), "body ID"),
        transform(member(value, "transform", "action"))};
  }
  if (kind == "set_body_type") {
    require_members(value, {"kind", "body_id", "body_kind"}, "body-type action");
    return SetBodyType{
        id(member(value, "body_id", "action"), "body ID"),
        body_kind(member(value, "body_kind", "action"))};
  }
  if (kind == "set_body_active") {
    require_members(value, {"kind", "body_id", "active"}, "body-active action");
    return SetBodyActive{
        id(member(value, "body_id", "action"), "body ID"),
        boolean(member(value, "active", "action"), "active")};
  }
  if (kind == "set_fixture_sensor") {
    require_members(value, {"kind", "fixture_id", "sensor"}, "fixture-sensor action");
    return SetFixtureSensor{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        boolean(member(value, "sensor", "action"), "sensor")};
  }
  if (kind == "set_fixture_material") {
    require_members(
        value,
        {"kind", "fixture_id", "friction_bits", "restitution_bits"},
        "fixture-material action");
    const auto friction = u32(member(value, "friction_bits", "action"), "friction bits");
    const auto restitution = u32(member(value, "restitution_bits", "action"), "restitution bits");
    require_nonnegative(friction, "friction bits");
    require_nonnegative(restitution, "restitution bits");
    return SetFixtureMaterial{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        friction,
        restitution};
  }
  if (kind == "set_fixture_filter") {
    require_members(value, {"kind", "fixture_id", "filter"}, "fixture-filter action");
    return SetFixtureFilter{
        id(member(value, "fixture_id", "action"), "fixture ID"),
        filter(member(value, "filter", "action"))};
  }
  if (kind == "set_fixture_density") {
    require_members(value, {"kind", "fixture_id", "density_bits"}, "fixture-density action");
    const auto density = u32(member(value, "density_bits", "action"), "density bits");
    require_nonnegative(density, "density bits");
    return SetFixtureDensity{
        id(member(value, "fixture_id", "action"), "fixture ID"), density};
  }
  if (kind == "reset_mass_data") {
    require_members(value, {"kind", "body_id"}, "reset-mass action");
    return ResetMassData{id(member(value, "body_id", "action"), "body ID")};
  }
  if (kind == "set_custom_mass_data") {
    require_members(
        value,
        {"kind", "body_id", "mass_bits", "center", "inertia_bits"},
        "custom-mass action");
    const auto mass = u32(member(value, "mass_bits", "action"), "mass bits");
    const auto inertia = u32(member(value, "inertia_bits", "action"), "inertia bits");
    require_finite(mass, "mass bits");
    require_nonnegative(inertia, "inertia bits");
    if (float_from_bits(mass) <= 0.0F) {
      throw std::runtime_error("custom mass must be positive");
    }
    return SetCustomMassData{
        id(member(value, "body_id", "action"), "body ID"),
        mass,
        vec2(member(value, "center", "action"), "mass center"),
        inertia};
  }
  if (kind == "step") {
    require_members(
        value,
        {"kind", "timestep_bits", "velocity_iterations", "position_iterations"},
        "step action");
    const auto timestep = u32(member(value, "timestep_bits", "action"), "timestep bits");
    const auto velocity = u32(member(value, "velocity_iterations", "action"), "velocity iterations");
    const auto position = u32(member(value, "position_iterations", "action"), "position iterations");
    require_finite(timestep, "timestep bits");
    if (float_from_bits(timestep) <= 0.0F || velocity == 0 || velocity > 255 ||
        position == 0 || position > 255) {
      throw std::runtime_error("step action is outside reviewed bounds");
    }
    return RigidStep{timestep, velocity, position};
  }
  if (kind == "destroy_fixture") {
    require_members(value, {"kind", "fixture_id"}, "destroy-fixture action");
    return DestroyFixture{id(member(value, "fixture_id", "action"), "fixture ID")};
  }
  if (kind == "destroy_body") {
    require_members(value, {"kind", "body_id"}, "destroy-body action");
    return DestroyBody{id(member(value, "body_id", "action"), "body ID")};
  }
  throw std::runtime_error("unsupported rigid-world action");
}

inline RigidExpectedCounts counts(const Json& value) {
  require_members(
      value,
      {"bodies", "fixtures", "contacts", "manifold_points", "events", "destructions"},
      "expected counts");
  return {
      u32(member(value, "bodies", "counts"), "body count"),
      u32(member(value, "fixtures", "counts"), "fixture count"),
      u32(member(value, "contacts", "counts"), "contact count"),
      u32(member(value, "manifold_points", "counts"), "manifold-point count"),
      u32(member(value, "events", "counts"), "event count"),
      u32(member(value, "destructions", "counts"), "destruction count")};
}

inline RigidContactIdentity contact_identity(const Json& value) {
  require_members(
      value,
      {"fixture_a_id", "child_a", "fixture_b_id", "child_b", "occurrence"},
      "contact identity");
  RigidContactIdentity result{
      id(member(value, "fixture_a_id", "contact identity"), "fixture A ID"),
      u32(member(value, "child_a", "contact identity"), "child A"),
      id(member(value, "fixture_b_id", "contact identity"), "fixture B ID"),
      u32(member(value, "child_b", "contact identity"), "child B"),
      u32(member(value, "occurrence", "contact identity"), "occurrence")};
  if (result.fixture_a_id == result.fixture_b_id || result.occurrence == 0) {
    throw std::runtime_error("invalid contact identity");
  }
  return result;
}

inline RigidWitnessFamily family(const Json& value) {
  const auto name = text(value, "witness family");
  if (name == "non_colliding_body_fixture_lifecycle") {
    return RigidWitnessFamily::non_colliding;
  }
  if (name == "single_contact_lifecycle") {
    return RigidWitnessFamily::single_contact;
  }
  throw std::runtime_error("unsupported witness family");
}

inline const std::vector<std::string_view>& required_witnesses(
    RigidWitnessFamily value) {
  static const std::vector<std::string_view> non_colliding{
      "static_body_created", "kinematic_body_created", "dynamic_body_created",
      "fixtures_created", "body_inspected", "fixture_inspected",
      "body_transform_changed", "body_type_changed", "body_deactivated",
      "body_reactivated", "sensor_enabled", "sensor_disabled",
      "material_changed", "filter_changed", "density_changed_without_mass_reset",
      "mass_reset", "custom_mass_set", "zero_contact_step", "fixture_destroyed",
      "body_destroyed"};
  static const std::vector<std::string_view> single_contact{
      "contact_created", "contact_begin", "contact_persisted", "manifold_active",
      "contact_solved", "warm_start_transferred", "sensor_touching",
      "sensor_without_manifold", "filter_removed_contact",
      "filter_recreated_contact", "deactivation_destroyed_contact",
      "reactivation_recreated_contact", "fixture_destroyed_contact",
      "body_cascade_end_ordered"};
  return value == RigidWitnessFamily::non_colliding ? non_colliding
                                                     : single_contact;
}

inline std::string action_name(const RigidAction& value) {
  static constexpr std::array<std::string_view, 16> names{
      "create_body", "create_fixture", "inspect_body", "inspect_fixture",
      "set_body_transform", "set_body_type", "set_body_active",
      "set_fixture_sensor", "set_fixture_material", "set_fixture_filter",
      "set_fixture_density", "reset_mass_data", "set_custom_mass_data", "step",
      "destroy_fixture", "destroy_body"};
  return std::string(names[value.index()]);
}

inline void validate_timeline(RigidTimeline& timeline) {
  if (timeline.bodies.empty() || timeline.bodies.size() > 64 ||
      timeline.fixtures.empty() || timeline.fixtures.size() > 128 ||
      timeline.actions.empty() || timeline.actions.size() > 64 ||
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
              std::is_same_v<T, ResetMassData> ||
              std::is_same_v<T, SetCustomMassData>) {
            if (!live_bodies.count(current.body_id)) {
              throw std::runtime_error("invalid rigid action order");
            }
          } else if constexpr (!std::is_same_v<T, RigidStep>) {
            if (!live_fixtures.count(current.fixture_id)) {
              throw std::runtime_error("invalid rigid action order");
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
  const auto required_actions = timeline.family == RigidWitnessFamily::non_colliding
                                    ? std::vector<std::string>{
                                          "create_body", "create_fixture", "inspect_body",
                                          "inspect_fixture", "set_body_transform", "set_body_type",
                                          "set_body_active", "set_fixture_sensor",
                                          "set_fixture_material", "set_fixture_filter",
                                          "set_fixture_density", "reset_mass_data",
                                          "set_custom_mass_data", "step", "destroy_fixture",
                                          "destroy_body"}
                                    : std::vector<std::string>{
                                          "create_body", "create_fixture", "set_body_active",
                                          "set_fixture_sensor", "set_fixture_filter", "step",
                                          "destroy_fixture", "destroy_body"};
  for (const auto& required : required_actions) {
    if (!action_kinds.count(required)) throw std::runtime_error("missing rigid action kind");
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

inline RigidWorldRequest decode(std::string_view record) {
  validate_bounded_json_record(record);
  const auto root = Json::parse(record.begin(), record.end());
  require_members(
      root,
      {"protocol_version", "record_kind", "request_id", "scenario_schema_version",
       "requested_trace_schema_version", "tolerance_profile_version",
       "tolerance_profile_sha256", "scenario"},
      "rigid-world request");
  if (u32(member(root, "protocol_version", "request"), "protocol version") != 1 ||
      text(member(root, "record_kind", "request"), "record kind") !=
          "rigid_world_request" ||
      u32(member(root, "scenario_schema_version", "request"), "scenario version") != 1 ||
      u32(member(root, "requested_trace_schema_version", "request"), "trace version") != 1 ||
      u32(member(root, "tolerance_profile_version", "request"), "tolerance version") != 1) {
    throw std::runtime_error("unsupported rigid-world protocol version");
  }
  const auto digest = text(
      member(root, "tolerance_profile_sha256", "request"), "tolerance digest");
  if (digest.size() != 64 ||
      !std::all_of(digest.begin(), digest.end(), [](unsigned char character) {
        return (character >= '0' && character <= '9') ||
               (character >= 'a' && character <= 'f');
      })) {
    throw std::runtime_error("invalid tolerance digest");
  }
  const auto& scenario = member(root, "scenario", "request");
  require_members(scenario, {"scenario_id", "source", "timelines"}, "rigid scenario");
  const auto& source = member(scenario, "source", "scenario");
  const auto source_kind = text(member(source, "kind", "source"), "source kind");
  if (source_kind == "named") {
    require_members(source, {"kind", "name"}, "named source");
    if (text(member(source, "name", "source"), "source name").empty()) {
      throw std::runtime_error("source name must not be empty");
    }
  } else if (source_kind == "seeded") {
    require_members(source, {"kind", "generator_id", "generator_version", "seed"}, "seeded source");
    static_cast<void>(id(member(source, "generator_id", "source"), "generator ID"));
    if (u32(member(source, "generator_version", "source"), "generator version") == 0) {
      throw std::runtime_error("generator version must be positive");
    }
    static_cast<void>(unsigned_value(member(source, "seed", "source"), "seed"));
  } else {
    throw std::runtime_error("unsupported source kind");
  }
  const auto& timelines = member(scenario, "timelines", "scenario");
  if (!timelines.is_array() || timelines.size() != 2) {
    throw std::runtime_error("rigid request must contain both witness families");
  }
  RigidWorldRequest request{
      id(member(root, "request_id", "request"), "request ID"),
      id(member(scenario, "scenario_id", "scenario"), "scenario ID"),
      {}};
  std::set<RigidWitnessFamily> families;
  std::size_t aggregate = 0;
  for (const auto& raw_timeline : timelines) {
    require_members(
        raw_timeline,
        {"witness_family", "bodies", "fixtures", "actions", "checkpoints"},
        "rigid timeline");
    RigidTimeline timeline;
    timeline.family = family(member(raw_timeline, "witness_family", "timeline"));
    if (!families.insert(timeline.family).second) {
      throw std::runtime_error("duplicate witness family");
    }
    const auto& bodies = member(raw_timeline, "bodies", "timeline");
    const auto& fixtures = member(raw_timeline, "fixtures", "timeline");
    const auto& actions = member(raw_timeline, "actions", "timeline");
    const auto& checkpoints = member(raw_timeline, "checkpoints", "timeline");
    if (!bodies.is_array() || !fixtures.is_array() || !actions.is_array() ||
        !checkpoints.is_array()) {
      throw std::runtime_error("rigid timeline collections must be arrays");
    }
    if (actions.size() > 64) {
      throw std::runtime_error("rigid action count outside reviewed bounds");
    }
    for (const auto& raw_body : bodies) {
      require_members(raw_body, {"body_id", "body_kind", "transform", "active"}, "body declaration");
      timeline.bodies.push_back({
          id(member(raw_body, "body_id", "body"), "body ID"),
          body_kind(member(raw_body, "body_kind", "body")),
          transform(member(raw_body, "transform", "body")),
          boolean(member(raw_body, "active", "body"), "body active")});
    }
    for (const auto& raw_fixture : fixtures) {
      require_members(
          raw_fixture,
          {"fixture_id", "owner_body_id", "shape", "density_bits", "friction_bits",
           "restitution_bits", "sensor", "filter"},
          "fixture declaration");
      const auto density = u32(member(raw_fixture, "density_bits", "fixture"), "density bits");
      const auto friction = u32(member(raw_fixture, "friction_bits", "fixture"), "friction bits");
      const auto restitution = u32(member(raw_fixture, "restitution_bits", "fixture"), "restitution bits");
      require_nonnegative(density, "density bits");
      require_nonnegative(friction, "friction bits");
      require_nonnegative(restitution, "restitution bits");
      timeline.fixtures.push_back({
          id(member(raw_fixture, "fixture_id", "fixture"), "fixture ID"),
          id(member(raw_fixture, "owner_body_id", "fixture"), "owner body ID"),
          shape(member(raw_fixture, "shape", "fixture")),
          density,
          friction,
          restitution,
          boolean(member(raw_fixture, "sensor", "fixture"), "fixture sensor"),
          filter(member(raw_fixture, "filter", "fixture"))});
    }
    for (const auto& raw_action : actions) {
      require_members(raw_action, {"action_id", "phase", "action"}, "action record");
      timeline.actions.push_back({
          id(member(raw_action, "action_id", "action record"), "action ID"),
          text(member(raw_action, "phase", "action record"), "action phase"),
          action(member(raw_action, "action", "action record"))});
    }
    for (const auto& raw_checkpoint : checkpoints) {
      require_members(
          raw_checkpoint,
          {"checkpoint_id", "after_action_id", "phase", "counts", "transitions"},
          "checkpoint");
      RigidCheckpoint checkpoint{
          id(member(raw_checkpoint, "checkpoint_id", "checkpoint"), "checkpoint ID"),
          id(member(raw_checkpoint, "after_action_id", "checkpoint"), "after-action ID"),
          text(member(raw_checkpoint, "phase", "checkpoint"), "checkpoint phase"),
          counts(member(raw_checkpoint, "counts", "checkpoint")),
          {}};
      const auto& transitions = member(raw_checkpoint, "transitions", "checkpoint");
      if (!transitions.is_array() || transitions.size() > 64) {
        throw std::runtime_error("transition count outside reviewed bounds");
      }
      for (const auto& raw_transition : transitions) {
        require_members(raw_transition, {"witness", "maybe_contact"}, "transition");
        RigidExpectedTransition transition;
        transition.witness = text(member(raw_transition, "witness", "transition"), "witness");
        const auto& maybe_contact = member(raw_transition, "maybe_contact", "transition");
        if (!maybe_contact.is_null()) {
          transition.maybe_contact = contact_identity(maybe_contact);
        }
        checkpoint.transitions.push_back(std::move(transition));
      }
      timeline.checkpoints.push_back(std::move(checkpoint));
    }
    aggregate += timeline.bodies.size() + timeline.fixtures.size() +
                 timeline.actions.size() + timeline.checkpoints.size();
    if (aggregate > 4096) {
      throw std::runtime_error("rigid request aggregate exceeds reviewed bounds");
    }
    validate_timeline(timeline);
    request.timelines.push_back(std::move(timeline));
  }
  if (!families.count(RigidWitnessFamily::non_colliding) ||
      !families.count(RigidWitnessFamily::single_contact)) {
    throw std::runtime_error("missing required witness family");
  }
  return request;
}

}  // namespace liquidfun::reference::rigid_world_decode
