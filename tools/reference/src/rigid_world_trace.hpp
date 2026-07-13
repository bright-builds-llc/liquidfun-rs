#pragma once

#include "protocol.hpp"
#include "rigid_world.hpp"

#include "nlohmann/json.hpp"

#include <Box2D/Box2D.h>

#include <cstdint>
#include <stdexcept>
#include <string>
#include <string_view>

namespace liquidfun::reference {

inline std::string_view rigid_body_kind_name(b2BodyType kind) {
  switch (kind) {
    case b2_staticBody: return "static";
    case b2_kinematicBody: return "kinematic";
    case b2_dynamicBody: return "dynamic";
  }
  throw std::runtime_error("unsupported pinned body kind");
}

inline nlohmann::json encode_rigid_vector(const b2Vec2& value) {
  return {{"x_bits", bits_from_float(value.x)},
          {"y_bits", bits_from_float(value.y)}};
}

inline nlohmann::json encode_rigid_transform(const b2Body& body) {
  return {{"position", encode_rigid_vector(body.GetPosition())},
          {"angle_bits", bits_from_float(body.GetAngle())}};
}

inline nlohmann::json encode_rigid_filter(const b2Filter& filter) {
  return {{"category_bits", filter.categoryBits},
          {"mask_bits", filter.maskBits},
          {"group_index", filter.groupIndex}};
}

inline nlohmann::json encode_rigid_counts(
    const RigidExpectedCounts& counts) {
  return {{"bodies", counts.bodies},
          {"fixtures", counts.fixtures},
          {"contacts", counts.contacts},
          {"manifold_points", counts.manifold_points},
          {"events", counts.events},
          {"destructions", counts.destructions}};
}

inline nlohmann::json encode_rigid_contact_identity(
    const RigidContactIdentity& identity) {
  return {{"fixture_a_id", identity.fixture_a_id},
          {"child_a", identity.child_a},
          {"fixture_b_id", identity.fixture_b_id},
          {"child_b", identity.child_b},
          {"occurrence", identity.occurrence}};
}

inline std::string_view rigid_family_name(RigidWitnessFamily family) {
  switch (family) {
    case RigidWitnessFamily::non_colliding:
      return "non_colliding_body_fixture_lifecycle";
    case RigidWitnessFamily::single_contact: return "single_contact_lifecycle";
    case RigidWitnessFamily::body_control: return "body_control_and_force_policy";
    case RigidWitnessFamily::island_warm_start:
      return "multi_contact_island_and_warm_start";
    case RigidWitnessFamily::sleeping_waking: return "sleeping_and_waking";
    case RigidWitnessFamily::continuous_collision:
      return "continuous_collision_and_sub_stepping";
    case RigidWitnessFamily::continuous_budget: return "continuous_budget_resume";
    case RigidWitnessFamily::query_ray: return "world_query_and_ray_cast";
    case RigidWitnessFamily::origin_shift: return "origin_shift_covariance";
  }
  throw std::runtime_error("unsupported rigid witness family");
}

inline std::string encode_rigid_world_result(const nlohmann::json& result) {
  return result.dump();
}

inline std::string encode_rigid_world_end(
    const std::string& request_id,
    std::uint32_t result_count,
    std::uint64_t reset_epoch) {
  return nlohmann::json{
      {"protocol_version", 1},
      {"record_kind", "rigid_world_end"},
      {"request_id", request_id},
      {"result_count", result_count},
      {"reset_epoch", reset_epoch},
      {"reset_verified", true}}
      .dump();
}

}  // namespace liquidfun::reference
