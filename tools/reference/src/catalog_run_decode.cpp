#include "catalog_run_decode.hpp"

#include "protocol.hpp"

#include <algorithm>
#include <cmath>
#include <iomanip>
#include <limits>
#include <sstream>
#include <stdexcept>

namespace liquidfun::reference::catalog_run_detail {
namespace {

constexpr std::size_t kMaximumResolvedBytes = 1024U * 1024U;
constexpr std::size_t kMaximumEntities = 4096U;
constexpr std::size_t kMaximumActions = 128U;
constexpr std::size_t kMaximumCheckpoints = 128U;
constexpr std::uint32_t kMaximumIterations = 1024U;

std::string as_sha256(const Json& value, std::string_view context) {
  if (!value.is_string()) {
    throw std::runtime_error(std::string(context) + " must be SHA-256");
  }
  const auto result = value.get<std::string>();
  if (result.size() != 64U ||
      !std::all_of(result.begin(), result.end(), [](char character) {
        return (character >= '0' && character <= '9') ||
               (character >= 'a' && character <= 'f');
      })) {
    throw std::runtime_error(std::string(context) + " must be lowercase hex");
  }
  return result;
}

void require_array_bound(
    const Json& value,
    std::size_t maximum,
    std::string_view context) {
  if (!value.is_array() || value.size() > maximum) {
    throw std::runtime_error(std::string(context) + " exceeds reviewed bound");
  }
}

}  // namespace

void require_members(
    const Json& value,
    std::initializer_list<std::string_view> expected,
    std::string_view context) {
  if (!value.is_object() || value.size() != expected.size()) {
    throw std::runtime_error(std::string(context) + " has invalid members");
  }
  for (const auto expected_member : expected) {
    if (!value.contains(expected_member)) {
      throw std::runtime_error(
          std::string(context) + " is missing required member");
    }
  }
}

const Json& member(
    const Json& value,
    std::string_view name,
    std::string_view context) {
  if (!value.is_object() || !value.contains(name)) {
    throw std::runtime_error(std::string(context) + " is missing member");
  }
  return value.at(name);
}

std::uint32_t as_u32(const Json& value, std::string_view context) {
  if (!value.is_number_unsigned()) {
    throw std::runtime_error(std::string(context) + " must be unsigned");
  }
  const auto result = value.get<std::uint64_t>();
  if (result > std::numeric_limits<std::uint32_t>::max()) {
    throw std::runtime_error(std::string(context) + " exceeds u32");
  }
  return static_cast<std::uint32_t>(result);
}

std::string as_id(const Json& value, std::string_view context) {
  if (!value.is_string()) {
    throw std::runtime_error(std::string(context) + " must be an ID");
  }
  const auto result = value.get<std::string>();
  const auto valid_character = [](char character) {
    return (character >= 'a' && character <= 'z') ||
           (character >= '0' && character <= '9') || character == '.' ||
           character == '_' || character == '-';
  };
  if (result.empty() || result.size() > kMaximumIdBytes ||
      !((result.front() >= 'a' && result.front() <= 'z') ||
        (result.front() >= '0' && result.front() <= '9')) ||
      !std::all_of(result.begin(), result.end(), valid_character)) {
    throw std::runtime_error(std::string(context) + " has invalid ID");
  }
  return result;
}

float as_finite_float(const Json& value, std::string_view context) {
  const auto result = float_from_bits(as_u32(value, context));
  if (!std::isfinite(result)) {
    throw std::runtime_error(std::string(context) + " must be finite");
  }
  return result;
}

b2Vec2 as_vec2(const Json& value, std::string_view context) {
  require_members(value, {"x_bits", "y_bits"}, context);
  return {as_finite_float(member(value, "x_bits", context), context),
          as_finite_float(member(value, "y_bits", context), context)};
}

CatalogRequest decode_request(
    std::string_view record,
    std::string_view actual_identity_sha256) {
  validate_bounded_json_record(record);
  const auto root = Json::parse(record);
  require_members(
      root,
      {"protocol_version", "record_kind", "request_id",
       "catalog_schema_version", "slug", "scenario_version",
       "generator_id", "generator_version", "maybe_seed", "settings",
       "resolved_bytes", "resolved_sha256", "provenance_requirements"},
      "catalog run request");
  if (as_u32(root.at("protocol_version"), "protocol version") != 1U ||
      root.at("record_kind") != "catalog_run_request" ||
      as_u32(root.at("catalog_schema_version"), "catalog schema version") !=
          1U ||
      as_u32(root.at("scenario_version"), "scenario version") != 1U ||
      as_u32(root.at("generator_version"), "generator version") != 1U) {
    throw std::runtime_error("unsupported catalog run version or kind");
  }
  const auto request_id = as_id(root.at("request_id"), "request ID");
  static_cast<void>(as_id(root.at("slug"), "catalog slug"));
  static_cast<void>(as_id(root.at("generator_id"), "generator ID"));
  if (!root.at("maybe_seed").is_null() &&
      !root.at("maybe_seed").is_number_unsigned()) {
    throw std::runtime_error("catalog seed must be null or unsigned");
  }
  require_members(
      root.at("settings"),
      {"timestep_bits", "velocity_iterations", "position_iterations",
       "particle_iterations"},
      "catalog settings");
  if (as_finite_float(root.at("settings").at("timestep_bits"), "timestep") <=
      0.0F) {
    throw std::runtime_error("catalog timestep must be positive");
  }
  for (const auto field : {"velocity_iterations", "position_iterations",
                           "particle_iterations"}) {
    const auto iterations = as_u32(root.at("settings").at(field), field);
    if (iterations == 0U || iterations > kMaximumIterations) {
      throw std::runtime_error("catalog iteration count is outside bounds");
    }
  }
  const auto resolved_sha256 =
      as_sha256(root.at("resolved_sha256"), "resolved SHA-256");
  require_array_bound(
      root.at("resolved_bytes"), kMaximumResolvedBytes, "resolved bytes");
  std::string resolved_bytes;
  resolved_bytes.reserve(root.at("resolved_bytes").size());
  for (const auto& byte : root.at("resolved_bytes")) {
    const auto value = as_u32(byte, "resolved byte");
    if (value > 255U) {
      throw std::runtime_error("resolved byte exceeds u8");
    }
    resolved_bytes.push_back(static_cast<char>(value));
  }
  if (sha256_hex(resolved_bytes) != resolved_sha256) {
    throw std::runtime_error("resolved bytes hash mismatch");
  }

  const auto payload = Json::parse(resolved_bytes);
  require_members(
      payload, {"identity", "entities", "actions", "checkpoints"},
      "resolved payload");
  require_members(
      payload.at("identity"),
      {"catalog_schema_version", "slug", "scenario_version", "generator_id",
       "generator_version", "maybe_seed", "settings"},
      "resolved identity");
  const auto& identity = payload.at("identity");
  if (identity.at("catalog_schema_version") !=
          root.at("catalog_schema_version") ||
      identity.at("slug") != root.at("slug") ||
      identity.at("scenario_version") != root.at("scenario_version") ||
      identity.at("generator_id") != root.at("generator_id") ||
      identity.at("generator_version") != root.at("generator_version") ||
      identity.at("maybe_seed") != root.at("maybe_seed") ||
      identity.at("settings") != root.at("settings")) {
    throw std::runtime_error("resolved identity mismatch");
  }
  require_array_bound(payload.at("entities"), kMaximumEntities, "entities");
  require_array_bound(payload.at("actions"), kMaximumActions, "actions");
  require_array_bound(
      payload.at("checkpoints"), kMaximumCheckpoints, "checkpoints");
  if (payload.at("actions").empty() || payload.at("checkpoints").empty()) {
    throw std::runtime_error("resolved schedule is incomplete");
  }
  for (std::size_t ordinal = 0; ordinal < payload.at("entities").size();
       ++ordinal) {
    const auto& entity = payload.at("entities").at(ordinal);
    require_members(entity, {"semantic_id", "scenario_id"}, "entity");
    const auto& semantic_id = entity.at("semantic_id");
    require_members(semantic_id, {"kind", "ordinal"}, "semantic entity ID");
    const auto kind = as_id(semantic_id.at("kind"), "semantic entity kind");
    if (kind != "body" && kind != "fixture" && kind != "joint" &&
        kind != "rope" && kind != "particle_system" &&
        kind != "particle_group" && kind != "particle") {
      throw std::runtime_error("unknown semantic entity kind");
    }
    if (as_u32(semantic_id.at("ordinal"), "semantic entity ordinal") !=
        ordinal) {
      throw std::runtime_error("semantic entity order is invalid");
    }
    auto wire_kind = kind;
    std::replace(wire_kind.begin(), wire_kind.end(), '_', '-');
    std::ostringstream expected_id;
    expected_id << "entity-" << wire_kind << '-' << std::setw(4)
                << std::setfill('0') << ordinal;
    if (as_id(entity.at("scenario_id"), "entity scenario ID") !=
        expected_id.str()) {
      throw std::runtime_error("semantic entity identity is invalid");
    }
  }

  const auto& provenance = root.at("provenance_requirements");
  require_members(
      provenance,
      {"required_identity_sha256", "limits_profile_sha256", "evidence_tier"},
      "provenance requirements");
  if (as_sha256(
          provenance.at("required_identity_sha256"), "required identity") !=
      actual_identity_sha256) {
    throw std::runtime_error("catalog identity requirement mismatch");
  }
  static_cast<void>(as_sha256(
      provenance.at("limits_profile_sha256"), "limits profile identity"));
  const auto tier = provenance.at("evidence_tier").get<std::string>();
  if (tier != "d0_replay" && tier != "d1_canonical" &&
      tier != "d2_supported" && tier != "d3_exploratory") {
    throw std::runtime_error("unknown evidence tier");
  }
  return {request_id, resolved_sha256, payload};
}

}  // namespace liquidfun::reference::catalog_run_detail
