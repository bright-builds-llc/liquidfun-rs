#include "rigid_world.hpp"

#include "protocol.hpp"
#include "rigid_world_decode.hpp"
#include "rigid_world_trace.hpp"
#include "rigid_world_phase8_execute.hpp"
#include "rigid_world_phase9_execute.hpp"
#include "rigid_world_phase10_execute.hpp"

#include <Box2D/Box2D.h>

#include <algorithm>
#include <limits>
#include <map>
#include <memory>
#include <set>
#include <stdexcept>
#include <string>
#include <tuple>
#include <type_traits>
#include <unordered_map>
#include <utility>

namespace liquidfun::reference {
namespace {

using Json = nlohmann::json;

b2Vec2 vector(RigidVec2Bits value) {
  return {float_from_bits(value.x), float_from_bits(value.y)};
}

b2BodyType body_type(RigidBodyKind kind) {
  switch (kind) {
    case RigidBodyKind::static_body: return b2_staticBody;
    case RigidBodyKind::kinematic_body: return b2_kinematicBody;
    case RigidBodyKind::dynamic_body: return b2_dynamicBody;
  }
  throw std::runtime_error("unreachable rigid body kind");
}

bool is_phase6_family(RigidWitnessFamily family) {
  return family == RigidWitnessFamily::non_colliding ||
         family == RigidWitnessFamily::single_contact;
}

bool should_wake(RigidWakePolicy policy) {
  return policy == RigidWakePolicy::wake;
}

class TimelineExecution;

class SemanticContactListener final : public b2ContactListener {
 public:
  explicit SemanticContactListener(TimelineExecution& execution)
      : execution_(execution) {}

  void BeginContact(b2Contact* contact) override;
  void EndContact(b2Contact* contact) override;
  void PreSolve(b2Contact* contact, const b2Manifold*) override;
  void PostSolve(b2Contact* contact, const b2ContactImpulse*) override;

 private:
  TimelineExecution& execution_;
};

#include "rigid_world/timeline.hpp"

}  // namespace


RigidVec2Bits semantic_phase8_reaction_force_bits(
    const b2Joint& joint,
    float inverse_timestep,
    bool solver_initialized) {
  const auto value = phase8_detail::semantic_reaction_force(
      joint, inverse_timestep, solver_initialized);
  return {bits_from_float(value.x), bits_from_float(value.y)};
}

bool phase8_reaction_guard_self_test() {
  b2World world({0.0F, 0.0F});
  b2BodyDef static_definition;
  auto* static_body = world.CreateBody(&static_definition);
  b2BodyDef dynamic_definition;
  dynamic_definition.type = b2_dynamicBody;
  dynamic_definition.position.Set(2.0F, 0.0F);
  auto* dynamic_body = world.CreateBody(&dynamic_definition);
  b2CircleShape circle;
  circle.m_radius = 0.25F;
  dynamic_body->CreateFixture(&circle, 1.0F);
  b2DistanceJointDef joint_definition;
  joint_definition.bodyA = static_body;
  joint_definition.bodyB = dynamic_body;
  joint_definition.length = 1.0F;
  joint_definition.frequencyHz = 1.0F;
  auto* joint = world.CreateJoint(&joint_definition);
  if (joint == nullptr) return false;
  const auto before = semantic_phase8_reaction_force_bits(*joint, 60.0F, false);
  world.Step(1.0F / 60.0F, 8, 3, 1);
  const auto after = semantic_phase8_reaction_force_bits(*joint, 60.0F, true);
  const auto exact_after = joint->GetReactionForce(60.0F);
  return before.x == 0U && before.y == 0U &&
         after.x == bits_from_float(exact_after.x) &&
         after.y == bits_from_float(exact_after.y) &&
         (after.x != 0U || after.y != 0U);
}

RigidWorldRequest decode_rigid_world_request(std::string_view record) {
  return rigid_world_decode::decode(record);
}

RigidWorldTrace RigidWorldAdapter::execute(std::string_view record) {
  const auto request = decode_rigid_world_request(record);
  Json timeline_results = Json::array();
  bool world_active = false;
  for (const auto& timeline : request.timelines) {
    {
      b2World world({0.0F, 0.0F});
      if (is_phase6_family(timeline.family)) {
        world.SetAllowSleeping(false);
        world.SetContinuousPhysics(false);
      }
      world_active = true;
      TimelineExecution execution(world, timeline);
      timeline_results.push_back(execution.run());
      if (world.GetBodyCount() != 0 || world.GetContactCount() != 0) {
        throw std::runtime_error("rigid request left pinned world state live");
      }
    }
    world_active = false;
  }
  for (const auto& raw_timeline : request.phase8_timelines) {
    {
      b2World world({0.0F, 0.0F});
      world_active = true;
      timeline_results.push_back(execute_phase8_timeline(world, raw_timeline));
      if (world.GetBodyCount() != 0 || world.GetJointCount() != 0 ||
          world.GetContactCount() != 0) {
        throw std::runtime_error("Phase 8 request left pinned world state live");
      }
    }
    world_active = false;
  }
  if (!request.phase9_timelines.empty()) {
    if (request.phase9_timelines.size() != timeline_results.size()) {
      throw std::runtime_error("Phase 9 timeline alignment failed");
    }
    for (std::size_t index = 0; index < request.phase9_timelines.size(); ++index) {
      const auto phase10_owns_timeline =
          !request.phase10_timelines.empty() &&
          !request.phase10_timelines.at(index).empty();
      if (!request.phase9_timelines.at(index).empty() && !phase10_owns_timeline) {
        apply_phase9_timeline(
            timeline_results.at(index), request.phase9_timelines.at(index));
      }
    }
  }
  if (!request.phase10_timelines.empty()) {
    if (request.phase10_timelines.size() != timeline_results.size()) {
      throw std::runtime_error("Phase 10 timeline alignment failed");
    }
    for (std::size_t index = 0; index < request.phase10_timelines.size(); ++index) {
      if (!request.phase10_timelines.at(index).empty()) {
        apply_phase10_timeline(
            timeline_results.at(index), request.phase10_timelines.at(index));
      }
    }
  }
  if (world_active) throw std::runtime_error("rigid world reset proof failed");
  if (reset_epoch_ == std::numeric_limits<std::uint64_t>::max()) {
    throw std::runtime_error("rigid world reset epoch overflowed");
  }
  ++reset_epoch_;
  Json result{
      {"protocol_version", 1},
      {"record_kind", "rigid_world_result"},
      {"request_id", request.request_id},
      {"trace_schema_version", 1},
      {"scenario_id", request.scenario_id},
      {"timelines", std::move(timeline_results)}};
  auto result_record = encode_rigid_world_result(result);
  auto end_record = encode_rigid_world_end(
      request.request_id, 1, reset_epoch_);
  if (result_record.size() + end_record.size() + 2 > kMaximumTraceBytes) {
    throw std::runtime_error("rigid world trace exceeds reviewed output limit");
  }
  return {
      std::move(result_record), std::move(end_record), reset_epoch_, true};
}

}  // namespace liquidfun::reference
