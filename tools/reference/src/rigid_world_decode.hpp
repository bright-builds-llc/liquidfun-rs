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
#include <utility>
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

inline RigidWakePolicy wake_policy(const Json& value) {
  const auto name = text(value, "wake policy");
  if (name == "wake") return RigidWakePolicy::wake;
  if (name == "preserve_sleep") return RigidWakePolicy::preserve_sleep;
  throw std::runtime_error("unsupported wake policy");
}

inline RigidFixtureChildSelector fixture_child_selector(const Json& value) {
  require_members(value, {"fixture_id", "child_index"}, "fixture child selector");
  return {
      id(member(value, "fixture_id", "selector"), "fixture ID"),
      u32(member(value, "child_index", "selector"), "child index")};
}

inline std::vector<RigidQueryRule> query_rules(const Json& value) {
  if (!value.is_array() || value.size() > kRigidWorldMaximumDirectives) {
    throw std::runtime_error("query directives exceed reviewed bounds");
  }
  std::vector<RigidQueryRule> result;
  std::set<std::pair<std::string, std::uint32_t>> selectors;
  for (const auto& raw : value) {
    require_members(raw, {"target", "directive"}, "query directive rule");
    const auto target = fixture_child_selector(member(raw, "target", "query rule"));
    if (!selectors.emplace(target.fixture_id, target.child_index).second) {
      throw std::runtime_error("duplicate query directive selector");
    }
    const auto name = text(member(raw, "directive", "query rule"), "query directive");
    const auto directive = name == "continue"
                               ? RigidQueryDirective::continue_query
                               : name == "terminate" ? RigidQueryDirective::terminate
                                                      : throw std::runtime_error("unsupported query directive");
    result.push_back({target, directive});
  }
  return result;
}

inline std::vector<RigidRayRule> ray_rules(const Json& value) {
  if (!value.is_array() || value.size() > kRigidWorldMaximumDirectives) {
    throw std::runtime_error("ray directives exceed reviewed bounds");
  }
  std::vector<RigidRayRule> result;
  std::set<std::pair<std::string, std::uint32_t>> selectors;
  for (const auto& raw : value) {
    require_members(raw, {"target", "directive"}, "ray directive rule");
    const auto target = fixture_child_selector(member(raw, "target", "ray rule"));
    if (!selectors.emplace(target.fixture_id, target.child_index).second) {
      throw std::runtime_error("duplicate ray directive selector");
    }
    const auto& raw_directive = member(raw, "directive", "ray rule");
    const auto kind = text(member(raw_directive, "kind", "ray directive"), "ray directive kind");
    RigidRayDirectiveValue directive;
    if (kind == "ignore") {
      require_members(raw_directive, {"kind"}, "ignore ray directive");
      directive.kind = RigidRayDirectiveKind::ignore;
    } else if (kind == "terminate") {
      require_members(raw_directive, {"kind"}, "terminate ray directive");
      directive.kind = RigidRayDirectiveKind::terminate;
    } else if (kind == "continue") {
      require_members(raw_directive, {"kind"}, "continue ray directive");
      directive.kind = RigidRayDirectiveKind::continue_ray;
    } else if (kind == "clip") {
      require_members(raw_directive, {"kind", "fraction_bits"}, "clip ray directive");
      directive.kind = RigidRayDirectiveKind::clip;
      directive.fraction = u32(member(raw_directive, "fraction_bits", "ray directive"), "clip fraction");
      const auto fraction = float_from_bits(directive.fraction);
      if (!std::isfinite(fraction) || fraction <= 0.0F || fraction > 1.0F) {
        throw std::runtime_error("ray clip fraction is outside reviewed bounds");
      }
    } else {
      throw std::runtime_error("unsupported ray directive");
    }
    result.push_back({target, directive});
  }
  return result;
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

#include "rigid_world_action_decode.hpp"
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
  if (name == "body_control_and_force_policy") return RigidWitnessFamily::body_control;
  if (name == "multi_contact_island_and_warm_start") return RigidWitnessFamily::island_warm_start;
  if (name == "sleeping_and_waking") return RigidWitnessFamily::sleeping_waking;
  if (name == "continuous_collision_and_sub_stepping") return RigidWitnessFamily::continuous_collision;
  if (name == "continuous_budget_resume") return RigidWitnessFamily::continuous_budget;
  if (name == "world_query_and_ray_cast") return RigidWitnessFamily::query_ray;
  if (name == "origin_shift_covariance") return RigidWitnessFamily::origin_shift;
  throw std::runtime_error("unsupported witness family");
}

#include "rigid_world_validate.hpp"
#include "rigid_world_phase8_decode.hpp"
inline RigidWorldRequest decode(std::string_view record) {
  validate_bounded_json_record(record);
  const auto root = Json::parse(record.begin(), record.end());
  require_members(
      root,
      {"protocol_version", "record_kind", "request_id", "scenario_schema_version",
       "requested_trace_schema_version", "tolerance_profile_version",
       "tolerance_profile_sha256", "scenario"},
      "rigid-world request");
  const auto& raw_timelines = member(
      member(root, "scenario", "request"), "timelines", "scenario");
  if (raw_timelines.is_array() && raw_timelines.size() == 19) {
    auto legacy_root = root;
    auto& legacy_timelines = legacy_root["scenario"]["timelines"];
    legacy_timelines.erase(legacy_timelines.begin() + 9, legacy_timelines.end());
    auto legacy_record = legacy_root.dump();
    legacy_record.push_back('\n');
    auto request = decode(legacy_record);
    request.phase8_timelines = decode_phase8_timelines(raw_timelines);
    return request;
  }
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
  if (!timelines.is_array() || timelines.size() < 2 || timelines.size() > 9) {
    throw std::runtime_error("rigid request timeline count is outside reviewed bounds");
  }
  RigidWorldRequest request{
      id(member(root, "request_id", "request"), "request ID"),
      id(member(scenario, "scenario_id", "scenario"), "scenario ID"),
      {},
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
    if (actions.size() > kRigidWorldMaximumActions) {
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
